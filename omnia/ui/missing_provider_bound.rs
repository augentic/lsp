trait Config {
    fn get(&self, key: &str) -> Option<String>;
}
trait StateStore {
    fn put(&self, key: &str, value: &[u8]);
}
trait Handler<P> {
    fn handle(self, provider: &P) -> u32;
}

struct Wasi;
impl Config for Wasi {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
}
impl StateStore for Wasi {
    fn put(&self, _key: &str, _value: &[u8]) {}
}

struct Concrete;
impl Handler<Wasi> for Concrete {
    fn handle(self, provider: &Wasi) -> u32 {
        provider.put("k", b"v");
        let _ = provider.get("k");
        0
    }
}

struct Generic;
impl<P: Config> Handler<P> for Generic {
    fn handle(self, provider: &P) -> u32 {
        let _ = provider.get("k");
        0
    }
}

fn main() {}
