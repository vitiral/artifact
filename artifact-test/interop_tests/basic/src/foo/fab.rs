//! This has foo stuff


/// #SPC-foo.yes
/// Do foo?
fn foo() {
}

#[test]
/// #TST-foo.yes4
fn test_samefile() {
    println!("TST-foo");  // nothing happens without `#`
}
