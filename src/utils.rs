use noise::utils::{NoiseImage, NoiseMap};
use rand::Rng;

#[allow(dead_code)]
pub fn write_example_to_file(map: &NoiseMap, filename: &str) {
    use std::{fs, path::Path};

    let target = Path::new("example_images/").join(Path::new(filename));

    fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
        .expect("Failed to create directories.");

    map.write_to_file(&target)
}

#[allow(dead_code)]
pub fn write_image_to_file(image: &NoiseImage, filename: &str) {
    use std::{fs, path::Path};

    let target = Path::new("example_images/").join(Path::new(filename));

    fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
        .expect("Failed to create directories.");

    image.write_to_file(&target)
}

#[allow(dead_code)]
pub fn rand_list_item<T: Copy>(list: &[T]) -> T {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..list.len());
    list[index]
}
