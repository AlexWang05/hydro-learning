use hydro_deploy::Deployment;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new(); // init hydro deployment

    // create flowbuilder -> store entire dataflow prog & manage its compilation
    let flow = hydro_lang::compile::builder::FlowBuilder::new();
    let leader = flow.process();
    let workers = flow.cluster();
    hydro_template::first_ten_cluster::first_ten_cluster(&leader, &workers);

    // flow.deploy() -> provisions dataflow prog on target machine
    let _nodes = flow
        .with_process(&leader, deployment.Localhost())
        .with_cluster(&workers, vec![deployment.Localhost(); 4])
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
