//! `unused_provider_bound` and `missing_provider_bound`: typed analysis
//! of provider trait bounds on `Handler` impls and helper functions.
//! Method calls resolve through typeck, so `Config::get` is never
//! confused with `StateStore::get`; usage walks local helper functions.

use std::collections::HashSet;

use clippy_utils::diagnostics::span_lint_and_help;
use rustc_hir as hir;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::def_id::{DefId, LocalDefId};
use rustc_hir::intravisit::{self, Visitor};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::ty::TypeckResults;
use rustc_span::Span;

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Flags a provider trait bound declared on a `Handler` impl or a
    /// helper function that no call path ever uses.
    ///
    /// ### Why is this bad?
    ///
    /// The bounds on a handler are its capability contract; an unused
    /// bound overstates what the handler needs.
    pub UNUSED_PROVIDER_BOUND,
    Deny,
    "provider trait bound that is declared but never used"
}

rustc_session::declare_lint! {
    /// ### What it does
    ///
    /// Flags a provider trait method used inside a `Handler` impl (or a
    /// bounded helper) without the matching provider bound.
    ///
    /// ### Why is this bad?
    ///
    /// Calls that resolve through a concrete provider type bypass the
    /// capability contract the bounds are supposed to state.
    pub MISSING_PROVIDER_BOUND,
    Deny,
    "provider trait method used without the matching bound"
}

pub struct ProviderBounds {
    handler: String,
    providers: Vec<String>,
}

rustc_session::impl_lint_pass!(ProviderBounds => [UNUSED_PROVIDER_BOUND, MISSING_PROVIDER_BOUND]);

/// One resolved provider-method usage inside a body.
struct Usage {
    trait_def: DefId,
    method: String,
    span: Span,
}

struct BodyScan<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    typeck: &'tcx TypeckResults<'tcx>,
    uses: Vec<Usage>,
    callees: Vec<DefId>,
}

/// The trait an associated fn belongs to, through either generic
/// dispatch or a concrete trait impl.
fn trait_of(cx: &LateContext<'_>, def_id: DefId) -> Option<DefId> {
    if let Some(trait_def) = cx.tcx.trait_of_assoc(def_id) {
        return Some(trait_def);
    }
    let impl_def = cx.tcx.trait_impl_of_assoc(def_id)?;
    Some(cx.tcx.impl_trait_id(impl_def))
}

impl<'tcx> Visitor<'tcx> for BodyScan<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        match expr.kind {
            hir::ExprKind::MethodCall(segment, ..) => {
                if let Some(def_id) = self.typeck.type_dependent_def_id(expr.hir_id)
                    && let Some(trait_def) = trait_of(self.cx, def_id)
                {
                    self.uses.push(Usage {
                        trait_def,
                        method: segment.ident.to_string(),
                        span: segment.ident.span,
                    });
                }
            }
            hir::ExprKind::Call(func, _) => {
                if let hir::ExprKind::Path(ref qpath) = func.kind {
                    match self.typeck.qpath_res(qpath, func.hir_id) {
                        Res::Def(DefKind::AssocFn, def_id) => {
                            if let Some(trait_def) = trait_of(self.cx, def_id) {
                                self.uses.push(Usage {
                                    trait_def,
                                    method: self.cx.tcx.item_name(def_id).to_string(),
                                    span: func.span,
                                });
                            }
                        }
                        Res::Def(DefKind::Fn, def_id) if def_id.is_local() => {
                            self.callees.push(def_id);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        intravisit::walk_expr(self, expr);
    }
}

impl ProviderBounds {
    pub fn new(conf: &crate::config::Conf) -> Self {
        Self {
            handler: conf.handler.clone(),
            providers: conf.providers.clone(),
        }
    }

    fn is_provider(&self, cx: &LateContext<'_>, def_id: DefId) -> bool {
        let name = cx.tcx.item_name(def_id);
        self.providers.iter().any(|provider| name.as_str() == provider)
    }

    /// Provider bounds declared in HIR generics, with their spans.
    fn declared(&self, cx: &LateContext<'_>, generics: &hir::Generics<'_>) -> Vec<(DefId, Span)> {
        let mut bounds = Vec::new();
        for predicate in generics.predicates {
            let hir::WherePredicateKind::BoundPredicate(bound_predicate) = predicate.kind else {
                continue;
            };
            for bound in bound_predicate.bounds {
                if let hir::GenericBound::Trait(poly) = bound
                    && let Some(def_id) = poly.trait_ref.trait_def_id()
                    && self.is_provider(cx, def_id)
                {
                    bounds.push((def_id, poly.trait_ref.path.span));
                }
            }
        }
        bounds
    }

    /// Provider bounds a callee declares, via its typed predicates.
    fn callee_bounds(&self, cx: &LateContext<'_>, def_id: DefId) -> Vec<DefId> {
        cx.tcx
            .predicates_of(def_id)
            .predicates
            .iter()
            .filter_map(|(clause, _)| {
                let trait_clause = clause.as_trait_clause()?;
                matches!(
                    trait_clause.self_ty().skip_binder().kind(),
                    rustc_middle::ty::TyKind::Param(_)
                )
                .then(|| trait_clause.def_id())
            })
            .filter(|def_id| self.is_provider(cx, *def_id))
            .collect()
    }

    fn scan(&self, cx: &LateContext<'_>, owner: LocalDefId) -> (Vec<Usage>, Vec<DefId>) {
        let Some(body) = cx.tcx.hir_maybe_body_owned_by(owner) else {
            return (Vec::new(), Vec::new());
        };
        let typeck = cx.tcx.typeck(owner);
        let mut scan = BodyScan {
            cx,
            typeck,
            uses: Vec::new(),
            callees: Vec::new(),
        };
        scan.visit_body(body);
        (scan.uses, scan.callees)
    }

    /// Every provider trait reachable from the given bodies: direct
    /// usages plus, transitively, local callees' usages and bounds.
    fn reachable(
        &self, cx: &LateContext<'_>, direct: &[Usage], callees: &[DefId],
    ) -> HashSet<DefId> {
        let mut used: HashSet<DefId> = direct.iter().map(|usage| usage.trait_def).collect();
        let mut visited: HashSet<DefId> = HashSet::new();
        let mut stack: Vec<DefId> = callees.to_vec();
        while let Some(callee) = stack.pop() {
            if !visited.insert(callee) {
                continue;
            }
            used.extend(self.callee_bounds(cx, callee));
            let Some(local) = callee.as_local() else { continue };
            let (uses, next) = self.scan(cx, local);
            used.extend(uses.iter().map(|usage| usage.trait_def));
            stack.extend(next);
        }
        used
    }

    fn check_bounds(
        &self, cx: &LateContext<'_>, subject: &str, declared: &[(DefId, Span)], direct: &[Usage],
        callees: &[DefId], report_missing: bool,
    ) {
        let used = self.reachable(cx, direct, callees);
        for (def_id, span) in declared {
            if !used.contains(def_id) {
                let name = cx.tcx.item_name(*def_id);
                span_lint_and_help(
                    cx,
                    UNUSED_PROVIDER_BOUND,
                    *span,
                    format!("provider bound `{name}` is never used by this {subject}"),
                    None,
                    "drop the bound, or delete the dead provider call path",
                );
            }
        }
        if !report_missing {
            return;
        }
        let declared_set: HashSet<DefId> = declared.iter().map(|(def_id, _)| *def_id).collect();
        let mut reported: HashSet<DefId> = HashSet::new();
        for usage in direct {
            if self.is_provider(cx, usage.trait_def)
                && !declared_set.contains(&usage.trait_def)
                && reported.insert(usage.trait_def)
            {
                let name = cx.tcx.item_name(usage.trait_def);
                span_lint_and_help(
                    cx,
                    MISSING_PROVIDER_BOUND,
                    usage.span,
                    format!(
                        "`{name}::{}` is used without a `{name}` bound on this {subject}",
                        usage.method
                    ),
                    None,
                    "declare the bound so the capability contract is explicit",
                );
            }
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for ProviderBounds {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        match item.kind {
            hir::ItemKind::Impl(impl_) => {
                let Some(of_trait) = impl_.of_trait else { return };
                let Some(trait_def) = of_trait.trait_ref.trait_def_id() else { return };
                if cx.tcx.item_name(trait_def).as_str() != self.handler {
                    return;
                }
                let declared = self.declared(cx, impl_.generics);
                let mut direct = Vec::new();
                let mut callees = Vec::new();
                for item_ref in impl_.items {
                    let (uses, next) = self.scan(cx, item_ref.owner_id.def_id);
                    direct.extend(uses);
                    callees.extend(next);
                }
                self.check_bounds(cx, "`Handler` impl", &declared, &direct, &callees, true);
            }
            hir::ItemKind::Fn { generics, .. } => {
                let declared = self.declared(cx, generics);
                if declared.is_empty() {
                    return;
                }
                let (direct, callees) = self.scan(cx, item.owner_id.def_id);
                self.check_bounds(cx, "function", &declared, &direct, &callees, true);
            }
            _ => {}
        }
    }
}
