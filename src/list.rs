pub trait List {
    fn list(&self)  -> Result<(), Box<dyn std::error::Error>>;
}