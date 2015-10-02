#![allow(unused_mut)]

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use doc::*;
use stepper::*;
use compose;
use normalize;


#[derive(Clone, Debug)]
pub struct AddStepper {
    pub head:Option<AddElement>,
    pub rest:Vec<AddElement>,
    pub stack:Vec<Vec<AddElement>>,
}

impl AddStepper {
    pub fn new(span:&AddSpan) -> AddStepper {
        let mut ret = AddStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<AddElement> {
        let res = self.head.clone();
        self.head = if self.rest.len() > 0 { Some(self.rest.remove(0)) } else { None };
        res
    }

    pub fn get_head(&self) -> AddElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.len() == 0
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        let span = match head {
            Some(AddGroup(_, ref span)) |
            Some(AddWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            },
            _ => {
                panic!("Entered wrong thing")
            }
        };
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }
}


#[derive(Clone, Debug)]
pub struct AddWriter {
    pub past:Vec<AddElement>,
    stack: Vec<Vec<AddElement>>,
}

impl AddWriter {
    pub fn new() -> AddWriter {
        AddWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(AddWithGroup(past));
    }

    pub fn close(&mut self, attrs: Attrs) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(AddGroup(attrs, past));
    }

    pub fn skip(&mut self, n: usize) {
        compose::add_place_any(&mut self.past, &AddSkip(n));
    }

    pub fn chars(&mut self, chars: &str) {
        compose::add_place_any(&mut self.past, &AddChars(chars.into()));
    }

    pub fn result(self) -> AddSpan {
        if self.stack.len() > 0 {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}



#[derive(Clone, Debug)]
pub struct DelWriter {
    pub past:Vec<DelElement>,
    stack: Vec<Vec<DelElement>>,
}

impl DelWriter {
    pub fn new() -> DelWriter {
        DelWriter {
            past: vec![],
            stack: vec![],
        }
    }

    pub fn begin(&mut self) {
        let past = self.past.clone();
        self.past = vec![];
        self.stack.push(past);
    }

    pub fn exit(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(DelWithGroup(past));
    }

    pub fn close(&mut self) {
        let past = self.past.clone();
        self.past = self.stack.pop().unwrap();
        self.past.push(DelGroup(past));
    }

    pub fn skip(&mut self, n: usize) {
        compose::del_place_any(&mut self.past, &DelSkip(n));
    }

    pub fn chars(&mut self, count: usize) {
        compose::del_place_any(&mut self.past, &DelChars(count));
    }

    pub fn result(self) -> DelSpan {
        if self.stack.len() > 0 {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}







#[derive(PartialEq, Clone)]
enum TrackType {
    NoType,
    TextBlock,
}

#[derive(Clone, Debug)]
struct Track {
    tag_a: Option<String>,
    tag_real: Option<String>,
    tag_b: Option<String>,
    is_original_a: bool,
    is_original_b: bool,
}

fn get_type(attrs:&Attrs) -> TrackType {
    TrackType::TextBlock    
}

struct Transform {
    tracks: Vec<Track>,
    a_del: DelWriter,
    a_add: AddWriter,
    b_del: DelWriter,
    b_add: AddWriter,
}

impl Transform {
    fn new() -> Transform {
        Transform {
            tracks: vec![],
            a_del: DelWriter::new(),
            a_add: AddWriter::new(),
            b_del: DelWriter::new(),
            b_add: AddWriter::new(),
        }
    }

    fn enter(&mut self, name:String) {
    //   iterA.apply(insrA);
    //   iterA.apply(insrB);
    //   delrA.enter();
    //   delrB.enter();
        self.tracks.push(Track {
            tag_a: Some(name.clone()),
            tag_real: Some(name.clone()),
            tag_b: Some(name.clone()),
            is_original_a: true,
            is_original_b: true,
        });

        self.a_del.begin();
        self.a_add.begin();
        self.b_del.begin();
        self.b_add.begin();
    }

    // Close the topmost track.
    fn abort(&mut self) -> (Option<String>, Option<String>, Option<String>) {
        let track = self.tracks.pop().unwrap();
        println!("ABORTIN {:?}", track);
        if let Some(ref real) = track.tag_real {
            // if track.tag_a.is_some() {
            self.a_del.close();
            self.a_add.close(container! { ("tag".into(), real.clone() )}); // fake

            self.b_del.close();
            self.b_add.close(container! { ("tag".into(), real.clone() )}); // fake
            // } else {
            //     self.a_add.close(container! { ("tag".into(), track.tag_a.into() )}); // fake
            // }
            // if (a) {
            //   insrA.alter(r, {}).close();
            // } else {
            //   insrA.close();
            // }
            // if (b) {
            //   insrB.alter(r, {}).close();
            // } else {
            //   insrB.close();
            // }
        }
        (track.tag_a, track.tag_real, track.tag_b)
    }

    fn skip_a(&mut self, n: usize) {
        self.a_del.skip(n);
        self.a_add.skip(n);
    }

    fn skip_b(&mut self, n: usize) {
        self.b_del.skip(n);
        self.b_add.skip(n);
    }

    fn chars_a(&mut self, chars: &str) {
        self.a_add.chars(chars);
    }

    fn chars_b(&mut self, chars: &str) {
        self.b_add.chars(chars);
    }

    fn current(&self) -> Option<Track> {
        let value = self.tracks.last();
        if let Some(track) = value {
            Some((*track).clone())
        } else {
            None
        }
    }

    fn close_a(&mut self) {
        let mut track = self.tracks.last_mut().unwrap();

        if track.is_original_a {
            self.a_del.exit();
        } else {
            self.a_del.close();
        }
        self.a_add.exit();

        self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });

        track.is_original_a = false;
        track.tag_a = None;
        track.tag_real = None;
    }

    fn close_b(&mut self) {
        let mut track = self.tracks.last_mut().unwrap();

        println!("CLOSES THE B {:?}", self.b_add);

        if track.is_original_b {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            self.b_del.close();
            self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });

        track.is_original_b = false;
        track.tag_b = None;
        track.tag_real = None;

      // var layer = schema.findType(tran.currentB()[2]);
      // var like = layer.like, unlike = layer.unlike;
      // var strategy = a == b ? like : unlike;

      // var track = tran.currentB(), a = track[0], r = track[1], b = track[2], origA = track[3], origB = track[4];

      // debugLog('  closeB():', a, r, b, like, unlike, origA, origB, demote);

      // tran.popB(like != 'split' || demote);

      // If client A is not open, or client A is open but we split along element bounds, close.
      // if (((!a && r) || (a && like == 'split')) && !demote) {
      //   if (origB) {
      //     // Preserve unmodified insertions.
      //     insrB.alter('', null);
      //   }
      //   insrB.leave();
      // }

      // If the other client is still open, we must switch to a close statement.
      // if ((a && like == 'split') && !demote) {
      //   insrA.alter(a, {}).close();
      // } else if ((!a && r) && !demote) {
      //   insrA.alter(r, {}).close();
      // }

      // if (!origB || (a && like == 'combine') || demote) {
      //   delrB.alter(b, {});
      // }
      // delrB.leave();
      
      // iterB.next();

      // if (demote) {
      //   tran.push(null, r, null);
      // }
      // if (tran.top() && !tran.top()[0] && !tran.top()[2] && tran.top()[1]) {
      //   insrA.close(); insrB.close(); tran.pop();
      // }
    }

    // Interrupt all tracks up the ancestry until we get to
    // a particular type, OR a type than could be an ancestor
    // of the given type
    fn interrupt(&mut self, itype:TrackType) {
        let mut regen = vec![];
        loop {
            let mut value = None;
            {
                if let Some(..) = self.current() {
                    value = self.current().clone();
                }
            }

            if let Some(track) = value {
                if track.tag_real.is_some() && false {
                    // schema.findType(tran.current()[1]) != type && schema.getAncestors(type).indexOf(schema.findType(tran.current()[1])) == -1
                    let aborted = self.abort();
                    regen.push(aborted);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        for group in regen {
            self.tracks.push(Track {
                tag_a: group.0,
                tag_real: None,
                tag_b: group.2,
                is_original_a: false,
                is_original_b: false,
            })
        }
    }

    fn regenerate(&mut self) {
        // okay do regen
        // Filter for types that are ancestors of the current type.
        // TODO
        for track in self.tracks.iter_mut() {
            if track.tag_real.is_none() {
                if track.tag_a.is_some() {
                    track.tag_real = track.tag_a.clone();
                    track.tag_a = track.tag_a.clone();
                    track.is_original_a = false;
                } else if track.tag_b.is_some() {
                    track.tag_real = track.tag_b.clone();
                    track.tag_a = track.tag_b.clone();
                    track.is_original_b = false;

                    // if (origA) {
                    //   insrA.enter();
                    // } else {
                    //   insrA.open(a || b, {});
                    // }
                    
                    // if (origB) {
                    //   insrB.enter();
                    // } else {
                    //   insrB.open(a || b, {});
                    // }
                }

                self.a_add.begin();
                self.b_add.begin();
            }
        }
    }

    fn result(mut self) -> (Op, Op) {
        let mut a_del = self.a_del;
        let mut a_add = self.a_add;
        let mut b_del = self.b_del;
        let mut b_add = self.b_add;

        for track in self.tracks.iter_mut().rev() {
            println!("TRACK RESULT: {:?}", track);
            if !track.is_original_a && track.tag_real.is_some() {
                // a_del.close();
                a_add.close(container! { ("tag".into(), track.tag_a.clone().unwrap() )});
            }
            if track.is_original_a {
                a_del.exit();
                a_add.exit();
            }
            if !track.is_original_b && track.tag_real.is_some() {
                // b_del.close();
                b_add.close(container! { ("tag".into(), track.tag_b.clone().unwrap() )});
            }
            if track.is_original_b {
                b_del.exit();
                b_add.exit();
            }
        }
        ((a_del.result(), a_add.result()), (b_del.result(), b_add.result()))
    }
}

fn transform_insertions(avec:&AddSpan, bvec:&AddSpan) -> (Op, Op) {
    // let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    let mut t = Transform::new();

    let mut a_type = TrackType::NoType;
    let mut b_type = TrackType::NoType;

    while !(a.is_done() && b.is_done()) {
        println!("FACED WITH {:?} {:?}", a.head, b.head);

        if a.is_done() || b.is_done() {
            println!("DONE ZO");
            t.regenerate();

            if a.is_done() {
                match b.head.clone() {
                    Some(AddChars(ref b_chars)) => {
                        t.chars_a(b_chars);
                        t.skip_b(b_chars.len());
                        b.next();
                    },
                    Some(AddSkip(b_count)) => {
                        t.skip_a(b_count);
                        t.skip_b(b_count);
                        b.next();
                    },
                    None => {
                        t.close_b();
                        b.exit();
                    },
                    _ => {
                        panic!("What");
                    }
                }
            }

        } else {
            match (a.head.clone(), b.head.clone()) {
                (Some(AddGroup(ref a_attrs, _)), Some(AddGroup(ref b_attrs, _))) => {
                    a_type = get_type(a_attrs);
                    b_type = get_type(b_attrs);

                    if a_type == b_type {
                        println!("My");
                    }

                    a.enter();
                    b.enter();
                    t.enter(a_attrs.get("tag").unwrap().clone())
                },
                (Some(AddSkip(a_count)), Some(AddSkip(b_count))) => {
                    if a_count > b_count {
                        a.head = Some(AddSkip(a_count - b_count));
                        b.next();
                    } else if a_count < b_count {
                        a.next();
                        b.head = Some(AddSkip(b_count - a_count));
                    } else {
                        a.next();
                        b.next();
                    }
                    t.skip_a(::std::cmp::min(a_count, b_count));
                    t.skip_b(::std::cmp::min(a_count, b_count));
                },
                (None, None) => {
                    a.exit();
                    b.exit();
                },
                (None, Some(AddSkip(b_count))) => {
                    t.interrupt(a_type.clone());
                    t.close_a();
                    a.exit();
                    println!("WHERE ARE WE WITH A {:?}", a);
                },
                (Some(AddSkip(a_count)), None) => {
                    t.interrupt(b_type.clone());
                    t.close_b();
                    // t.closeA()
                    b.exit()
                },
                (Some(AddChars(ref a_chars)), _) => {
                    t.skip_a(a_chars.len());
                    t.chars_b(a_chars);
                    a.next();
                },
                _ => {
                    panic!("No idea: {:?}, {:?}", a.head, b.head);
                },
            }
        }
    }

    let (op_a, op_b) = t.result();
    println!("RESULT A: {:?}", op_a.clone());
    println!("RESULT B: {:?}", op_b.clone());
    (op_a, op_b)
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
