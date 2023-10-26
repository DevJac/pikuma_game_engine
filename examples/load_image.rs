fn main() {
    let tree: image::DynamicImage = image::io::Reader::open("assets/images/tree.png")
        .unwrap()
        .decode()
        .unwrap();
    dbg!(tree.width(), tree.height());

    let bullet: image::DynamicImage = image::io::Reader::open("assets/images/bullet.png")
        .unwrap()
        .decode()
        .unwrap();
    dbg!(bullet.width(), bullet.height());
}
