use arbitrary::Arbitrary;
use nutype::nutype;

#[nutype(derive(Debug, Arbitrary))]
struct Wrapper<T>(Vec<T>);

fn main() {
    fn gen(bytes: &[u8]) -> Wrapper<bool> {
        let mut u = arbitrary::Unstructured::new(bytes);
        Wrapper::<bool>::arbitrary(&mut u).unwrap()
    }
    assert_eq!(gen(&[]).into_inner(), vec![]);
    assert_eq!(gen(&[1]).into_inner(), vec![false]);
    assert_eq!(gen(&[1, 3, 5]).into_inner(), vec![true, false]);
}
