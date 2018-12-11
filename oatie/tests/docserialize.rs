use serde_json;
use oatie::*;
use oatie::doc::*;

#[test]
fn test_docserialize_ron() {
    let start = doc_span![
        DocChars("birds snakes and aeroplanes")
    ];
    let res = ron::ser::to_string(&start).unwrap();
    println!("re {:?}", res);
    let res2: Vec<DocElement>  = ron::de::from_str(&res).unwrap();
    println!("re {:?}", res2);
    assert_eq!(start, res2);
    // assert_eq!(res, "[DocChars(\"birds snakes and aeroplanes\"),]");
    eprintln!();

    // FIXME links don't work in serialization
    let start = doc_span![
        DocChars("birds snakes and aeroplanes", { Style::Bold => None /*, Style::Link => Some("Wow".to_string()) */ })
    ];
    let res = ron::ser::to_string(&start).unwrap();
    println!("re {:?}", res);
    let res2: Vec<DocElement>  = ron::de::from_str(&res).unwrap();
    assert_eq!(start, res2);
    println!("re {:?}", res2);
    // assert_eq!(res, "[DocChars((\"birds snakes and aeroplanes\",[Bold,],)),]");
    eprintln!();

    let input = r#"[DocGroup({"tag":"h1",},[DocChars(["dsdfsdno",],[Normie,],),],)]"#;
    let res: Vec<DocElement>  = ron::de::from_str(&input).unwrap();

    let input = r#"[DocChars("dsdfsdno"),]"#;
    let res: Vec<DocElement>  = ron::de::from_str(&input).unwrap();
}

#[test]
fn test_docserialize_json() {
    let start = doc_span![
        DocChars("birds snakes and aeroplanes")
    ];
    let res = serde_json::to_string(&start).unwrap();
    println!("re.....: {:?}", res);


    let res2: Vec<DocElement>  = serde_json::from_str(&res).unwrap();
    println!("re {:?}", res2);
    assert_eq!(start, res2);
    eprintln!();




    // FIXME links don't work in serialization
    let start = doc_span![
        DocChars("birds snakes and aeroplanes", { Style::Bold => None /*, Style::Link => Some("Wow".to_string()) */ })
    ];
    let res = serde_json::to_string(&start).unwrap();
    println!("re {:?}", res);
    let res2: Vec<DocElement>  = serde_json::from_str(&res).unwrap();
    assert_eq!(start, res2);
    println!("re {:?}", res2);
    eprintln!();

}