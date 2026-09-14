fn main() {
    let _ = eros::error!("{{}");
    let _ = eros::error!("}}{");
    let _ = eros::error!("{missing}");
}
