use dag_statistics::DirectedAcyclicGraph;
use std::fs::File;
use std::env::args;

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let args = args();
    let filename = args.skip(1).next().expect("Expected a filename argument");
    let file = File::open(&filename)?;

    let dag = DirectedAcyclicGraph::from_read(file)?;

    println!("AVG DAG DEPTH: {:.2}", dag.avg_depth());
    println!("AVG NODES PER DEPTH: {:.2}", dag.avg_node_per_depth());
    println!("AVG REF: {:.3}", dag.avg_ref());
    println!("AVG OUT REF: {:.3}", dag.avg_out_ref());
    println!("MAX DEPTH: {}", dag.max_depth());

    Ok(())
}
