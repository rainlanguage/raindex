fn main() {
    cynic_codegen::register_schema("raindex")
        .from_sdl_file("schema/raindex.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
