mod config;

pub fn build() {
    let folder = "clusters";
    let deployments = config::deployments();
    let files = deployments.iter().map(|deployment| {
        let path = std::path::Path::new(&folder)
            .join(deployment.config.stage.to_string())
            .join("resources.yaml");
        let contents = deployment.resources().join("\n---\n");
        (path, contents)
    });

    let _ = std::fs::remove_dir_all(&folder);

    files.for_each(|(path, contents)| {
        std::fs::create_dir_all(path.clone().parent().unwrap()).unwrap();
        std::fs::write(path, {
            let mut res = contents;
            res.push('\n');
            res
        })
        .unwrap();
    });
}
