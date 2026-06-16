use key_paths_core::FieldDiff;
use key_paths_derive::{FieldDiff as FieldDiffDerive, Kp};

#[derive(Kp, Hash, FieldDiffDerive, PartialEq, Debug)]
struct App {
    count: i32,
    label: String,
}

#[test]
fn field_diff_tracks_per_field_hashes() {
    let app = App {
        count: 1,
        label: "a".into(),
    };
    let mut fields = Vec::new();
    app.field_hashes(&mut fields);
    assert_eq!(fields.len(), 2);
    assert!(fields.iter().any(|(p, _)| *p == AppField::Count));
    assert!(fields.iter().any(|(p, _)| *p == AppField::Label));
}
