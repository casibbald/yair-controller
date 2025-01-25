use crate::controllers::lib::kubecontroller;
use kube::CustomResourceExt;
fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&kubecontroller::Document::crd()).unwrap()
    )
}
