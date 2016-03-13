#![allow(unused_mut)]

extern crate oatie;
extern crate term_painter;
#[macro_use] extern crate literator;
extern crate log;
#[macro_use] extern crate env_logger;

use oatie::doc::*;
use oatie::compose;
use oatie::normalize;
use oatie::transform::*;

fn test_start() {
    let _ = env_logger::init();
}

#[test]
fn test_transform_goose() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)])
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)])
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)])
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());

    println!("what {:?}", b_);
    // assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());

    //TODO this is a bug.
    // println!("why {:?}", compose::compose(&(vec![], vec![AddSkip(6)]), &(b_.0, b_.1)));
}

#[test]
fn test_transform_gander() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)])
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)])
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(6)]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_cory() {
    let a = vec![
        AddSkip(1), AddChars("1".into())
    ];
    let b = vec![
        AddSkip(1), AddChars("2".into())
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddSkip(1), AddChars("12".into()),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_wheat() {
    let a = vec![
        AddSkip(12), AddChars("_".into())
    ];
    let b = vec![
        AddSkip(5), AddChars("D".into())
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddSkip(5), AddChars("D".into()), AddSkip(7), AddChars("_".into())
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_rice() {
    let a = vec![
        AddSkip(1), AddChars("a".into())
    ];
    let b = vec![
        AddSkip(2), AddChars("c".into())
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddSkip(1), AddChars("a".into()), AddSkip(1), AddChars("c".into())
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_bacon() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(5)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(5)]),
    ];
    let b = vec![
        AddSkip(11), AddChars("_".into())
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(5)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(5)]),
        AddSkip(1), AddChars("_".into()),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_berry() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![AddSkip(15)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(15)]),
    ];
    let b = vec![
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![AddSkip(15)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(15)]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_brown() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(5)]),
    ];
    let b = vec![
        AddSkip(2),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(2)]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(4)]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(1)]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_sonic() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(30)]),
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![AddSkip(30)]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(30)]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_tails() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(15)]),
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![AddSkip(30)]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![AddSkip(15)]),
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![AddSkip(15)]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_snippet() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(15)
            ])
        ]),
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(15)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddGroup(container! { ("tag".into(), "p".into()) }, vec![
                    AddSkip(15)
                ]),
            ])
        ]),
    ]);

    assert_eq!(normalize(compose::compose(&(vec![], a), &a_)), res.clone());
    assert_eq!(normalize(compose::compose(&(vec![], b), &b_)), res.clone());
}

#[test]
fn test_transform_anthem() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(10)
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(10)
        ]),
    ];
    let b = vec![
        AddSkip(5),
        AddGroup(container! { ("tag".into(), "b".into()) }, vec![
            AddSkip(10)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(5),
            AddGroup(container! { ("tag".into(), "b".into()) }, vec![
                AddSkip(5),
            ]),
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddGroup(container! { ("tag".into(), "b".into()) }, vec![
                AddSkip(5),
            ]),
            AddSkip(5),
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_yellow() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(5)
            ])
        ]),
    ];
    let b = vec![
        AddSkip(3),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(2)
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(3)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(3),
                AddGroup(container! { ("tag".into(), "p".into()) }, vec![
                    AddSkip(2)
                ]),
            ])
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(3)
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_black() {
    // TODO revert back to things with li's
    let a = vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(5)
            ])
        ]),
    ];
    let b = vec![
        AddSkip(2),
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(2)
            ])
        ]),
    ];

    println!("HERE IS A: {:?}", a);
    println!("HERE IS B: {:?}", b);

    let (a_, b_) = transform_insertions(&a, &b);

    println!("lol");

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(2)
            ]),
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(2)
            ]),
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(1)
            ])
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_ferociously() {
    let a = vec![
        AddGroup(container! { ("tag".into(), "h1".into()) }, vec![
            AddSkip(8)
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(5)
        ]),
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "h3".into()) }, vec![
            AddSkip(8)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_tony() {
    let a = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                ]),
            ])
        ]),
        AddGroup(container! { ("tag".into(), "p".into()) }, vec![
            AddSkip(5)
        ]),
    ];
    let b = vec![
        AddGroup(container! { ("tag".into(), "h3".into()) }, vec![
            AddSkip(8)
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_drone() {
    let a = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(4),
                    AddChars("a".into()),
                ]),
            ])
        ]),
    ];
    let b = vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(4),
                    AddChars("b".into()),
                ]),
            ])
        ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b), &b_));
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_feedback() {
    let a = vec![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(1),
                    AddGroup(container! { ("tag".into(), "b".into()) }, vec![AddSkip(3)]),
                ]),
        //     ])
        // ]),
    ];
    let b = vec![
        // AddWithGroup(vec![
        //     AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(2),
                    AddGroup(container! { ("tag".into(), "b".into()) }, vec![AddSkip(3)]),
                ]),
        //     ])
        // ]),
    ];

    let (a_, b_) = transform_insertions(&a, &b);
    
    let a_res = normalize(compose::compose(&(vec![], a.clone()), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    assert_eq!(a_res.1, b_res.1); // TODO fix the normalize case for deletes??
    

    let (b_, a_) = transform_insertions(&b, &a);

    let a_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
    let b_res = normalize(compose::compose(&(vec![], a.clone()), &a_));
    assert_eq!(a_res.1, b_res.1); // TODO fix the normalize case for deletes??
}

#[test]
fn test_transform_dawn() {
    let a = vec![
        DelSkip(2),
        DelChars(1),
    ];
    let b = vec![
        DelSkip(2),
        DelChars(1),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelSkip(2),
        DelChars(1),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_zone() {
    let a = vec![
        DelSkip(1),
        DelChars(1),
    ];
    let b = vec![
        DelSkip(2),
        DelChars(1),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelSkip(1),
        DelChars(2),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}

#[test]
fn test_transform_everyday() {
    let a = vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ];
    let b = vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ];

    let (a_, b_) = transform_deletions(&a, &b);

    let res = (vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ], vec![]);

    let a_res = normalize(compose::compose(&(a, vec![]), &(a_, vec![])));
    let b_res = normalize(compose::compose(&(b, vec![]), &(b_, vec![])));

    println!("A : {:?}", a_res);
    println!("B : {:?}", b_res);
    println!("r : {:?}", res);

    assert_eq!(a_res, res.clone());
    assert_eq!(b_res, res.clone());
}


#[test]
fn test_transform_pick() {
    let a = (vec![
        DelWithGroup(vec![
            DelGroup(vec![]),
        ]),
    ], vec![]);
    let b = (vec![], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddChars("hi".into()),
            ]),
        ]),
    ]);

    let (a_, b_) = transform(&a, &b);

    // let res = (vec![
    //     DelWithGroup(vec![
    //         DelGroup(vec![]),
    //     ]),
    // ], vec![]);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));

    // println!("A : {:?}", a_res);
    // println!("B : {:?}", b_res);
    // println!("r : {:?}", res);
    // 
    // assert_eq!(a_res, res.clone());
    // assert_eq!(b_res, res.clone());
    
    assert_eq!(a_res, b_res);
}

#[test]
fn test_transform_hot() {
    let a = (vec![
        DelWithGroup(vec![
            DelWithGroup(vec![
                DelGroup(vec![
                    DelSkip(6),
                ]),
            ]),
        ]),
    ], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddGroup(container! { ("tag".into(), "p".into())}, vec![
                    AddChars("hi".into()),
                    AddSkip(6),
                ]),
            ]),
        ]),
    ]);
    let b = (vec![], vec![
        AddWithGroup(vec![
            AddWithGroup(vec![
                AddWithGroup(vec![
                    AddSkip(6),
                    AddChars("a".into()),
                ]),
            ]),
        ]),
    ]);

    let (a_, b_) = transform(&a, &b);

    let a_res = normalize(compose::compose(&a, &a_));
    let b_res = normalize(compose::compose(&b, &b_));
    
    println!("");
    println!("A' {:?}", a_res);
    println!("B' {:?}", b_res);
    println!("");
    
    assert_eq!(a_res, b_res);
}
