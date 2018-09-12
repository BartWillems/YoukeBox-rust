extern crate youkebox;

use youkebox::player::duration_to_seconds;

#[test]
fn parse_duration() {
    assert_eq!(4210, duration_to_seconds(&"PT1H10M10S".to_string()));
}
