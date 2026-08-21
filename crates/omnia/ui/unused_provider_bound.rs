trait Config {
    fn get(&self, key: &str) -> Option<String>;
}
trait StateStore {
    fn put(&self, key: &str, value: &[u8]);
}
trait Publisher {
    fn send(&self, message: &str);
}
trait Handler<P> {
    fn handle(self, provider: &P) -> u32;
}

struct Direct;
impl<P: Config + StateStore> Handler<P> for Direct {
    fn handle(self, provider: &P) -> u32 {
        let _ = provider.get("KEY");
        0
    }
}

fn notify<P: Publisher>(provider: &P) {
    provider.send("hello");
}

struct Delegating;
impl<P: Config + Publisher> Handler<P> for Delegating {
    fn handle(self, provider: &P) -> u32 {
        let _ = provider.get("KEY");
        notify(provider);
        0
    }
}

fn helper_with_unused<P: Config + Publisher>(provider: &P) {
    provider.send("only publisher");
}

fn main() {
    let wide = Direct;
    let _ = wide;
}
