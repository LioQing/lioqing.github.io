#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use image;
    use std::fs;
    use std::path::Path;
    use walkdir::WalkDir;

    const FROM_PATH: &str = "./images";
    const TO_PATH: &str = "./assets/images";
    const PROJECT_COVERS: &str = "project-covers";
    const RESIZE_SIZE: u32 = 512;
    const EXCLUDE_FILES: &[&str] = &["logo-light.png", "logo-dark.png"];

    fn resize_and_save_image(source: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let img = image::open(source)?;

        let resized = img.resize(
            RESIZE_SIZE,
            RESIZE_SIZE,
            image::imageops::FilterType::Lanczos3,
        );

        resized.save(dest)?;

        Ok(())
    }

    fs::create_dir_all(TO_PATH).expect("create destination directory");

    for entry in WalkDir::new(FROM_PATH) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Error accessing entry: {e}");
                continue;
            }
        };

        if entry.file_type().is_dir() {
            let rel_path = entry.path().strip_prefix(FROM_PATH).unwrap();
            let dest_path = Path::new(TO_PATH).join(rel_path);
            fs::create_dir_all(&dest_path).expect("create subdirectory");
            continue;
        }

        if let Some(file_name) = entry.file_name().to_str() {
            if EXCLUDE_FILES.contains(&file_name) {
                println!("Skipping excluded file: {file_name}");
                continue;
            }
        }

        let rel_path = entry.path().strip_prefix(FROM_PATH).unwrap();
        let dest_path = Path::new(TO_PATH).join(rel_path);

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }

        let needs_resize = entry.path().to_string_lossy().contains(PROJECT_COVERS);

        if needs_resize {
            match resize_and_save_image(entry.path(), &dest_path) {
                Ok(_) => println!("Resized and saved: {dest_path:?}"),
                Err(e) => eprintln!("Error processing image {:?}: {}", entry.path(), e),
            }
        } else {
            match fs::copy(entry.path(), &dest_path) {
                Ok(_) => println!("Copied: {dest_path:?}"),
                Err(e) => eprintln!("Error copying {:?}: {}", entry.path(), e),
            }
        }
    }

    println!("Image assets copying completed!");
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("unsupported target architecture: wasm32, this is a native only binary");
}
