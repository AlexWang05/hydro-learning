use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();
    // create flowbuilder -> store entire dataflow prog & manage its compilation

    let flow = hydro_lang::compile::builder::FlowBuilder::new();
    let process = flow.process();
    hydro_template::first_ten::first_ten(&process);

    // flow.deploy() -> provisions dataflow prog on target machine
    // returns _nodes, a struct w/ handles to instantiated machines
    let _nodes = flow
        .with_process(&process, deployment.Localhost())
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
