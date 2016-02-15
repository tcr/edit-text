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
        self.past.place(&AddSkip(n));
    }

    pub fn chars(&mut self, chars: &str) {
        self.past.place(&AddChars(chars.into()));
    }

    pub fn group(&mut self, attrs: &Attrs, span: &AddSpan) {
        self.past.place(&AddGroup(attrs.clone(), span.clone()));
    }

    pub fn with_group(&mut self, span: &AddSpan) {
        self.past.place(&AddWithGroup(span.clone()));
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
        self.past.place(&DelSkip(n));
    }

    pub fn chars(&mut self, count: usize) {
        self.past.place(&DelChars(count));
    }

    pub fn result(self) -> DelSpan {
        if self.stack.len() > 0 {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
    }
}



trait Named {
    fn get_name(&self) -> Option<String>;
}

impl Named for Attrs {
    fn get_name(&self) -> Option<String> {
        match self.get("tag") {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}






#[derive(PartialEq, Clone, Debug)]
enum TrackType {
    NoType,
    Lists,
    ListItems,
    BlockQuotes,
    Blocks,
    BlockObjects,
    Inlines,
    InlineObjects,
}

fn get_tag_type(tag: &str) -> Option<TrackType> {
    match tag {
        "ul" => Some(TrackType::Lists),
        "li" => Some(TrackType::ListItems),
        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some(TrackType::Blocks),
        "b" => Some(TrackType::Inlines),
        _ => None,
    }
}

fn get_type(attrs: &Attrs) -> Option<TrackType> {
    match attrs.get_name() {
        Some(tag) => get_tag_type(&tag[..]),
        _ => None
    }
}

impl TrackType {
    fn parents(&self) -> Vec<TrackType> {
        use transform::TrackType::*;
        match *self {
            Lists => vec![ListItems, BlockQuotes],
            ListItems => vec![Lists],
            BlockQuotes => vec![ListItems, BlockQuotes],
            Blocks => vec![ListItems, BlockObjects],
            BlockObjects => vec![ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![Blocks],
            _ => { panic!("this shouldnt be"); }
        }
    }

    fn ancestors(&self) -> Vec<TrackType> {
        use transform::TrackType::*;
        match *self {
            Lists => vec![Lists, ListItems, BlockQuotes],
            ListItems => vec![Lists, ListItems, BlockQuotes],
            BlockQuotes => vec![Lists, ListItems, BlockQuotes],
            Blocks => vec![Lists, ListItems, BlockObjects],
            BlockObjects => vec![Lists, ListItems, BlockQuotes],
            Inlines | InlineObjects => vec![Lists, ListItems, BlockQuotes, Blocks],
            _ => { panic!("this shouldnt be"); }
        }
    }
}





#[derive(Clone, Debug)]
struct Track {
    tag_a: Option<String>,
    tag_real: Option<String>,
    tag_b: Option<String>,
    is_original_a: bool,
    is_original_b: bool,
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
        let last = self.tracks.iter().rposition(|x| x.tag_real.is_some()).and_then(|x| Some(x + 1)).unwrap_or(0);

    //   iterA.apply(insrA);
    //   iterA.apply(insrB);
    //   delrA.enter();
    //   delrB.enter();
        self.tracks.insert(last, Track {
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

    fn enter_a(&mut self, a: String, b: Option<String>) {
        let last = self.tracks.iter().rposition(|x| x.tag_real.is_some()).and_then(|x| Some(x + 1)).unwrap_or(0);

        self.tracks.insert(last, Track {
            tag_a: Some(a.clone()),
            tag_real: Some(a.clone()),
            tag_b: b.clone(),
            is_original_a: true,
            is_original_b: b.is_some(),
        });

        self.a_del.begin();
        self.a_add.begin();
        if b.is_some() {
            self.b_del.begin();
        }
        self.b_add.begin();
    }

    fn enter_b(&mut self, a: Option<String>, b: String) {
        let last = self.tracks.iter().rposition(|x| x.tag_real.is_some()).and_then(|x| Some(x + 1)).unwrap_or(0);

        self.tracks.insert(last, Track {
            tag_a: a.clone(),
            tag_real: Some(b.clone()),
            tag_b: Some(b.clone()),
            is_original_a: a.is_some(),
            is_original_b: true,
        });

        if a.is_some() {
            self.a_del.begin();
        }
        self.a_add.begin();
        self.b_del.begin();
        self.b_add.begin();
    }

    // Close the topmost track.
    fn abort(&mut self) -> (Option<String>, Option<String>, Option<String>) {
        let track = self.tracks.pop().unwrap();

        println!("aborting: {:?}", track);
        if let Some(ref real) = track.tag_real {
            // if track.tag_a.is_some() {
            //     self.a_del.close();
            // }
            self.a_add.close(container! { ("tag".into(), real.clone() )}); // fake

            // if track.tag_b.is_some() {
            //     self.b_del.close();
            // }
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

    fn unenter_a(&mut self) {
        self.a_del.begin();
        let track = self.tracks.last_mut().unwrap();
        track.tag_a = track.tag_real.clone();
    }

    fn unenter_b(&mut self) {
        self.b_del.begin();
        let track = self.tracks.last_mut().unwrap();
        track.tag_b = track.tag_real.clone();
    }

    fn skip_a(&mut self, n: usize) {
        self.a_del.skip(n);
        self.a_add.skip(n);
    }

    fn skip_b(&mut self, n: usize) {
        self.b_del.skip(n);
        self.b_add.skip(n);
    }

    fn group_a(&mut self, attrs: &Attrs, span: &AddSpan) {
        self.a_add.group(attrs, span);
    }

    fn group_b(&mut self, attrs: &Attrs, span: &AddSpan) {
        self.b_add.group(attrs, span);
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

    fn close(&mut self) {
        let (mut track, index) = self.top_track_a();

        if track.is_original_a && track.tag_real == track.tag_a {
            self.a_del.exit();
            self.a_add.exit();
        } else {
            self.a_del.close();
            self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        if track.is_original_b && track.tag_real == track.tag_b {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            self.b_del.close();
            self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        self.tracks.remove(index);
    }

    fn top_track_a(&mut self) -> (Track, usize) {
        let index = self.tracks.iter().rposition(|x| x.tag_a.is_some()).unwrap();
        (self.tracks[index].clone(), index)
    }

    fn top_track_b(&mut self) -> (Track, usize) {
        let index = self.tracks.iter().rposition(|x| x.tag_b.is_some()).unwrap();
        (self.tracks[index].clone(), index)
    }

    fn close_a(&mut self) {
        println!("TRACKS CLOSE A: {:?}", self.tracks);
        let (mut track, index) = self.top_track_a();

        println!("CLOSES THE A {:?}", self.a_del);
        println!("CLOSES THE A {:?}", self.a_add);

        if track.is_original_a { // && track.tag_real == track.tag_a {
            self.a_del.exit();
            self.a_add.exit();
        } else {
            self.a_del.close();
            self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        // if track.is_original_b {
        //     self.b_del.close();
        // }
        self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });

        if track.tag_b.is_none() {
            self.tracks.remove(index);
        } else {
            self.tracks[index].is_original_a = false;
            self.tracks[index].is_original_b = false;
            self.tracks[index].tag_a = None;
            self.tracks[index].tag_real = None;
        }
    }

    fn close_b(&mut self) {
        println!("TRACKS CLOSE B: {:?}", self.tracks);
        let (mut track, index) = self.top_track_b();

        println!("CLOSES THE B {:?}", self.b_del);
        println!("CLOSES THE B {:?}", self.b_add);

        if track.is_original_b { // && track.tag_real == track.tag_b {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            println!("1");
            self.b_del.close();
            self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
            println!("2");
        }

        // if track.is_original_a {
        //     self.a_del.close();
        // }
        self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });

        if track.tag_a.is_none() {
            self.tracks.remove(index);
        } else {
            self.tracks[index].is_original_a = false;
            self.tracks[index].is_original_b = false;
            self.tracks[index].tag_b = None;
            self.tracks[index].tag_real = None;
        }
    }

    // Interrupt all tracks up the ancestry until we get to
    // a particular type, OR a type than could be an ancestor
    // of the given type
    fn interrupt(&mut self, itype:TrackType) {
        let mut regen = vec![];
        loop {
            if let Some(track) = self.current() {
                if track.tag_real.is_some() && {
                    let tag_type = get_tag_type(&track.tag_real.unwrap()).unwrap();
                    tag_type != itype && tag_type.ancestors().iter().position(|x| *x == itype).is_some()
                } {
                    // schema.findType(tran.current()[1]) != type && schema.getAncestors(type).indexOf(schema.findType(tran.current()[1])) == -1
                    println!("aborting by {:?}", itype);
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
                println!("REGENERATE: {:?}", track);
                if track.tag_b.is_some() {
                    track.tag_real = track.tag_b.clone();
                    // track.tag_a = track.tag_b.clone();
                    // track.is_original_b = false;

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
                if track.tag_a.is_some() {
                    track.tag_real = track.tag_a.clone();
                    // track.tag_a = track.tag_a.clone();
                    // track.is_original_a = false;
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
                a_add.close(container! { ("tag".into(), track.tag_a.clone().unwrap() )});
            }
            if track.is_original_a {
                a_del.exit();
                a_add.exit();
            }
            if !track.is_original_b && track.tag_real.is_some() {
                b_add.close(container! { ("tag".into(), track.tag_b.clone().unwrap() )});
            }
            if track.is_original_b {
                b_del.exit();
                b_add.exit();
            }
        }
        ((a_del.result(), a_add.result()), (b_del.result(), b_add.result()))
    }

    fn current_type(&self) -> Option<TrackType> {
        // TODO
        // self.tracks.last().unwrap().
        let attrs: Attrs = container! {
            ("tag".to_string(), self.tracks.last().clone().unwrap().tag_real.clone().unwrap() )
        };
        get_type(&attrs)
    }
}

pub fn transform_insertions(avec:&AddSpan, bvec:&AddSpan) -> (Op, Op) {
    // let mut res = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    let mut t = Transform::new();

    while !(a.is_done() && b.is_done()) {
        if a.is_done() {
            println!("tracks {:?}", t.tracks);
            t.regenerate();
            println!("A IS DONE: {:?}", b.head.clone());

            println!("WHAT IS UP {:?}", t.b_add);
            println!("`````` tracks {:?}", t.tracks);
            
            match b.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_b(1);
                    t.group_a(attrs, span);
                    b.next();
                },
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
                    panic!("What: {:?}", b.head);
                }
            }
        } else if b.is_done() {
            t.regenerate();
            println!("B IS DONE: {:?}", a.head.clone());

            match a.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_a(1);
                    t.group_b(attrs, span);
                    a.next();
                },
                Some(AddChars(ref a_chars)) => {
                    t.skip_a(a_chars.len());
                    t.chars_b(a_chars);
                    a.next();
                },
                Some(AddSkip(a_count)) => {
                    t.skip_a(a_count);
                    t.skip_b(a_count);
                    a.next();
                },
                None => {
                    t.close_a();
                    a.exit();
                },
                _ => {
                    panic!("Unknown value: {:?}", a.head.clone());
                }
            }

        } else {
            match (a.head.clone(), b.head.clone()) {
                // Closing
                (None, None) => {
                    let (a_tag, b_tag) = {
                        let t = t.tracks.last().unwrap();
                        (t.tag_a.clone(), t.tag_b.clone())
                    };

                    if a_tag.is_some() && b_tag.is_some() && get_tag_type(&a_tag.clone().unwrap()[..]) == get_tag_type(&b_tag.clone().unwrap()[..]) {
                        // t.interrupt(a_tag || b_tag);
                        a.exit();
                        b.exit();
                        t.close();
                    } else if a_tag.is_some() && (b_tag.is_none() || get_tag_type(&a_tag.clone().unwrap()[..]).unwrap().ancestors().iter().position(|x| *x == get_tag_type(&b_tag.clone().unwrap()[..]).unwrap()).is_some()) {
                        // t.interrupt(a_tag);
                        a.exit();
                        t.close_a();
                    } else if b_tag.is_some() {
                        // t.interrupt(b_tag);
                        b.exit();
                        t.close_b();
                    }
                },
                (None, _) => {
                    let a_typ = get_tag_type(&t.tracks.iter().rev().find(|t| t.tag_a.is_some()).unwrap().tag_a.clone().unwrap()[..]).unwrap();
                    println!("what is up with a {:?}", t.a_add);
                    t.interrupt(a_typ);
                    // println!("... {:?} {:?}", t.a_del, t.a_add);
                    // println!("... {:?} {:?}", t.b_del, t.b_add);
                    println!("~~~> tracks {:?}", t.tracks);
                    t.close_a();
                    // println!("...");
                    a.exit();
                    println!("<~~~ tracks {:?}", t.tracks);
                    // println!("WHERE ARE WE WITH A {:?}", a);
                },
                (_, None) => {
                    // println!("... {:?} {:?}", t.a_del, t.a_add);
                    // println!("... {:?} {:?}", t.b_del, t.b_add);
                    let b_typ = get_tag_type(&t.tracks.iter().rev().find(|t| t.tag_b.is_some()).unwrap().tag_b.clone().unwrap()[..]).unwrap();
                    t.interrupt(b_typ);
                    t.close_b();
                    // t.closeA()
                    b.exit();
                },

                // Opening
                (Some(AddGroup(ref a_attrs, _)), Some(AddGroup(ref b_attrs, _))) => {
                    let a_type = get_type(a_attrs).unwrap();
                    let b_type = get_type(b_attrs).unwrap();

                    println!("groupgruop {:?} {:?}", a_type, b_type);
                    if a_type == b_type {
                        a.enter();
                        b.enter();
                        if a_attrs.get_name() == b_attrs.get_name() {
                            t.enter(a_attrs.get_name().unwrap());
                        } else {
                            t.enter_a(a_attrs.get_name().unwrap(), b_attrs.get_name());
                        }
                    } else if get_type(b_attrs).unwrap().ancestors().iter().position(|x| *x == get_type(a_attrs).unwrap()).is_some() {
                        a.enter();
                        t.enter_a(a_attrs.get_name().unwrap(), None);
                    } else {
                        b.enter();
                        t.enter_b(None, b_attrs.get_name().unwrap());
                    }

                    // TODO if they are different tags THEN WHAT

                },
                (Some(AddGroup(ref a_attrs, _)), _) => {
                    a.enter();
                    t.enter_a(a_attrs.get_name().unwrap(), None);
                    println!("TRACKS: {:?}", t.tracks);
                },
                (_, Some(AddGroup(ref b_attrs, _))) => {
                    // println!("groupgruop {:?} {:?}", a_type, b_type);
                    // t.regenerate();
                    b.enter();
                    let b_type = get_type(b_attrs);

                    if t.current_type() == b_type {
                        t.unenter_b();
                    } else {
                        t.enter_b(None, b_attrs.get_name().unwrap());
                    }
                },

                // Rest
                (Some(AddSkip(a_count)), Some(AddSkip(b_count))) => {
                    t.regenerate();

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
                (Some(AddSkip(a_count)), Some(AddChars(ref b_chars))) => {
                    t.regenerate();

                    b.next();
                    t.chars_a(b_chars);
                    t.skip_b(b_chars.len());
                },
                (Some(AddChars(ref a_chars)), _) => {
                    t.regenerate();

                    t.skip_a(a_chars.len());
                    t.chars_b(a_chars);
                    a.next();
                },

                // ???
                _ => {
                    panic!("No idea: {:?}, {:?}", a.head, b.head);
                },
            }
        }
    }

    println!("TRACK A DEL {:?}", t.a_del);
    println!("TRACK A ADD {:?}", t.a_add);
    println!("TRACK B DEL {:?}", t.b_del);
    println!("TRACK B ADD {:?}", t.b_add);

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

    let (a_, b_) = transform_insertions(&a, &b);

    let res = (vec![], vec![
        AddGroup(container! { ("tag".into(), "ul".into()) }, vec![
            AddGroup(container! { ("tag".into(), "li".into()) }, vec![
                AddSkip(5)
            ])
        ]),
    ]);

    let a_res = normalize(compose::compose(&(vec![], a), &a_));
    let b_res = normalize(compose::compose(&(vec![], b.clone()), &b_));
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
