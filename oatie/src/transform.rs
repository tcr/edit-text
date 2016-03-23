#![allow(unused_mut)]

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use doc::*;
use stepper::*;
use compose;
use normalize;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;


#[derive(Clone, Debug)]
pub struct DelStepper {
    pub head:Option<DelElement>,
    pub rest:Vec<DelElement>,
    pub stack:Vec<Vec<DelElement>>,
}

impl DelStepper {
    pub fn new(span:&DelSpan) -> DelStepper {
        let mut ret = DelStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<DelElement> {
        let res = self.head.clone();
        self.head = if self.rest.len() > 0 { Some(self.rest.remove(0)) } else { None };
        res
    }

    pub fn get_head(&self) -> DelElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.len() == 0
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        let span = match head {
            Some(DelGroup(ref span)) |
            Some(DelWithGroup(ref span)) => {
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

    pub fn place_all(&mut self, span: &AddSpan) {
        self.past.place_all(span);
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

    pub fn group(&mut self, span: &DelSpan) {
        self.past.place(&DelGroup(span.clone()));
    }

    pub fn with_group(&mut self, span: &DelSpan) {
        self.past.place(&DelWithGroup(span.clone()));
    }

    pub fn place_all(&mut self, span: &DelSpan) {
        self.past.place_all(span);
    }

    pub fn result(self) -> DelSpan {
        if self.stack.len() > 0 {
            println!("{:?}", self);
            assert!(false, "cannot get result when stack is still full");
        }
        self.past
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

fn get_tag_type<T: TagLike>(tag: T) -> Option<TrackType> {
    let tag = tag.tag();
    if let Some(tag) = tag {
        match tag.as_ref() {
            "ul" => Some(TrackType::Lists),
            "li" => Some(TrackType::ListItems),
            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => Some(TrackType::Blocks),
            "b" => Some(TrackType::Inlines),
            _ => None,
        }
    } else {
        None
    }
}

trait TagLike {
    fn tag(&self) -> Option<String>;
}

impl TagLike for String {
    fn tag(&self) -> Option<String> {
        Some(self.clone())
    }
}

impl<'a> TagLike for &'a String {
    fn tag(&self) -> Option<String> {
        Some((*self).clone())
    }
}

impl<'a> TagLike for &'a str {
    fn tag(&self) -> Option<String> {
        Some((*self).into())
    }
}

impl TagLike for Attrs {
    fn tag(&self) -> Option<String> {
        match self.get("tag") {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

impl<'a> TagLike for &'a Attrs {
    fn tag(&self) -> Option<String> {
        match self.get("tag") {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

impl TagLike for Option<String> {
    fn tag(&self) -> Option<String> {
        self.clone()
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
        let last = self.tracks.iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

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
        let last = self.tracks.iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

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
        let last = self.tracks.iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

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
        let track = self.next_track_b().unwrap();
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

    fn with_group_a(&mut self, span: &AddSpan) {
        self.a_add.with_group(span);
    }

    fn with_group_b(&mut self, span: &AddSpan) {
        self.b_add.with_group(span);
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
        let index = self.tracks.iter()
            .rposition(|x| x.tag_a.is_some())
            .unwrap();
        (self.tracks[index].clone(), index)
    }
    
    fn next_track_a(&mut self) -> Option<&mut Track> {
        if let Some(index) = self.tracks.iter().position(|x| x.tag_a.is_none()) {
            Some(&mut self.tracks[index])
        } else {
            None
        }
    }
    
    fn next_track_a_type(&mut self) -> Option<TrackType> {
        if let Some(track) = self.next_track_a() {
            get_tag_type(track.tag_real.clone())
        } else {
            None
        }
    }

    fn top_track_b(&mut self) -> (Track, usize) {
        let index = self.tracks.iter()
            .rposition(|x| x.tag_b.is_some())
            .unwrap();
        (self.tracks[index].clone(), index)
    }
    
    fn next_track_b(&mut self) -> Option<&mut Track> {
        if let Some(index) = self.tracks.iter().position(|x| x.tag_b.is_none()) {
            Some(&mut self.tracks[index])
        } else {
            None
        }
    }
    
    fn next_track_b_type(&mut self) -> Option<TrackType> {
        if let Some(track) = self.next_track_b() {
            get_tag_type(track.tag_real.clone())
        } else {
            None
        }
    }

    fn close_a(&mut self) {
        println!("TRACKS CLOSE A: {:?}", self.tracks);
        let (mut track, index) = self.top_track_a();
        
        // Determine whether to split tags for this track type.
        // TODO do the same for track opening?
        let track_split = if let Some(tag) = track.tag_real.clone() {
            get_tag_type(&tag) != Some(TrackType::Lists) && get_tag_type(&tag) != Some(TrackType::Inlines)
        } else {
            true
        };

        if track.is_original_a && (track_split || track.tag_b.is_none()) { // && track.tag_real == track.tag_a {
            self.a_del.exit();
            self.a_add.exit();
        } else {
            self.a_del.close();
            if track_split || track.tag_b.is_none() {
                self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
            }
        }

        // if track.is_original_b {
        //     self.b_del.close();
        // }
        println!("CLOSES THE B {:?}", self.b_add);
        if track_split || track.tag_b.is_none() {
            self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        if track.tag_b.is_none() {
            self.tracks.remove(index);
        } else {
            self.tracks[index].is_original_a = false;
            if track_split {
                self.tracks[index].is_original_b = false;
            }
            self.tracks[index].tag_a = None;
            if track_split {
                self.tracks[index].tag_real = None;
            }
        }
        
        println!("A ADD NOW {:?}", self.a_add);
    }

    fn close_b(&mut self) {
        println!("close_b:");
        for t in &self.tracks {
            println!(" - {:?}", t);
        }
        
        let (mut track, index) = self.top_track_b();

        println!("CLOSES THE B {:?}", self.b_del);
        println!("CLOSES THE B {:?}", self.b_add);

        // Determine whether to split tags for this track type.
        // TODO do the same for track opening?
        // NOTE i might have done this already
        let track_split = if let Some(tag) = track.tag_real.clone() {
            get_tag_type(&tag) != Some(TrackType::Lists) && get_tag_type(&tag) != Some(TrackType::Inlines)
        } else {
            true
        };

        if track.is_original_b && (track_split || track.tag_a.is_none()) { // && track.tag_real == track.tag_b {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            println!("1");

            self.b_del.close();
            if track_split || track.tag_a.is_none() {
                self.b_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
            }
            println!("2 {:?}", self.b_del);
        }

        // if track.is_original_a {
        //     self.a_del.close();
        // }
        if track_split || track.tag_a.is_none() {
            self.a_add.close(container! { ("tag".into(), track.tag_real.clone().unwrap().into()) });
        }

        if track.tag_a.is_none() {
            self.tracks.remove(index);
        } else {
            if track_split {
                self.tracks[index].is_original_a = false;
            }
            self.tracks[index].is_original_b = false;
            self.tracks[index].tag_b = None;
            if track_split {
                self.tracks[index].tag_real = None;
            }
        }
    }

    // Interrupt all tracks up the ancestry until we get to
    // a particular type, OR a type than could be an ancestor
    // of the given type
    fn interrupt(&mut self, itype: TrackType, inclusive: bool) {
        let mut regen = vec![];
        loop {
            if let Some(track) = self.current() {
                let (istag, hasparent) = if let Some(ref real) = track.tag_real {
                    println!("WOW {:?} {:?}", real, itype);
                    let tag_type = get_tag_type(real).unwrap();
                    (tag_type == itype, tag_type.ancestors().iter().position(|x| *x == itype).is_some())
                } else {
                    (false, false)
                };
                if track.tag_real.is_some() && ((!istag && hasparent) || (istag && inclusive)) {
                    // schema.findType(tran.current()[1]) != type && schema.getAncestors(type).indexOf(schema.findType(tran.current()[1])) == -1
                    println!("aborting by {:?} {:?} {:?}", itype, inclusive, istag);
                    let aborted = self.abort();
                    regen.push(aborted);
                    if istag && inclusive {
                        break;
                    }
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

                // This only happens when opening split elements.
                // if !track.is_original_a {
                    self.a_add.begin();
                // }
                // if !track.is_original_b {
                    self.b_add.begin();
                // }
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
        get_tag_type(&attrs)
    }
}

pub fn transform_insertions(avec:&AddSpan, bvec:&AddSpan) -> (Op, Op) {
    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    let mut t = Transform::new();

    while !(a.is_done() && b.is_done()) {
        println!("{}", Green.bold().paint("Tracks:"));
        for t in &t.tracks {
            println!("{}", BrightGreen.paint(format!(" - {:?}", t)));
        }
        
        println!("{}", BrightGreen.paint(format!(" @ a_del: {:?}", t.a_del)));
        println!("{}", BrightGreen.paint(format!(" @ a_add: {:?}", t.a_add)));
        println!("{}", BrightGreen.paint(format!(" @ b_del: {:?}", t.b_del)));
        println!("{}", BrightGreen.paint(format!(" @ b_add: {:?}", t.b_add)));

        if a.is_done() {
            // println!("tracks {:?}", t.tracks);
            t.regenerate();
            println!("{}", BrightYellow.paint(format!("Finishing B: {:?}", b.head.clone())));

            match b.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_b(1);
                    t.group_a(attrs, span);
                    b.next();
                },
                Some(AddWithGroup(ref span)) => {
                    t.skip_b(1);
                    t.with_group_a(span);
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
            }
        } else if b.is_done() {
            t.regenerate();
            println!("{}", BrightYellow.paint(format!("Finishing A: {:?}", a.head.clone())));

            match a.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_a(1);
                    t.group_b(attrs, span);
                    a.next();
                },
                Some(AddWithGroup(ref span)) => {
                    t.skip_a(1);
                    t.with_group_b(span);
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
            }

        } else {
            println!("{}", BrightYellow.paint(format!("Next step: {:?}", (a.head.clone(), b.head.clone()))));

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
                (None, Some(AddChars(ref b_chars))) => {
                    t.chars_a(b_chars);
                    t.skip_b(b_chars.len());
                    b.next();
                }
                (None, _) => {
                    let a_typ = get_tag_type(&t.tracks.iter().rev().find(|t| t.tag_a.is_some()).unwrap().tag_a.clone().unwrap()[..]).unwrap();
                    println!("what is up with a {:?}", t.a_add);
                    t.interrupt(a_typ, false);
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
                    t.interrupt(b_typ, false);
                    t.close_b();
                    // t.closeA()
                    b.exit();
                },

                // Opening
                (Some(AddGroup(ref a_attrs, _)), Some(AddGroup(ref b_attrs, _))) => {
                    let a_type = get_tag_type(a_attrs).unwrap();
                    let b_type = get_tag_type(b_attrs).unwrap();

                    println!("groupgruop {:?} {:?}", a_type, b_type);
                    if a_type == b_type {
                        a.enter();
                        b.enter();
                        if a_attrs.tag() == b_attrs.tag() {
                            t.enter(a_attrs.tag().unwrap());
                        } else {
                            t.enter_a(a_attrs.tag().unwrap(), b_attrs.tag());
                        }
                    } else if get_tag_type(b_attrs).unwrap().ancestors().iter().position(|x| *x == get_tag_type(a_attrs).unwrap()).is_some() {
                        a.enter();
                        t.enter_a(a_attrs.tag().unwrap(), None);
                    } else {
                        b.enter();
                        t.enter_b(None, b_attrs.tag().unwrap());
                    }

                    // TODO if they are different tags THEN WHAT

                },
                (Some(AddGroup(ref a_attrs, _)), _) => {
                    a.enter();
                    let a_type = get_tag_type(a_attrs);

                    if t.next_track_a_type() == a_type {
                        if a_type == Some(TrackType::ListItems) {
                            println!("INTERRUPTING");
                            t.interrupt(a_type.unwrap(), true);
                            if let Some(j) = t.next_track_a() {
                                j.tag_a = a_attrs.tag();
                                j.is_original_a = true;
                            }
                            t.a_del.begin();
                        } else {
                            t.unenter_a();
                        }
                    } else {
                        t.enter_a(a_attrs.tag().unwrap(), None);
                    }
                    
                    // println!("adding left group:");
                    // for t in &t.tracks {
                    //     println!(" - {:?}", t);
                    // }
                },
                (_, Some(AddGroup(ref b_attrs, _))) => {
                    // println!("groupgruop {:?} {:?}", a_type, b_type);
                    // t.regenerate();
                    b.enter();
                    let b_type = get_tag_type(b_attrs);

                    if t.next_track_b_type() == b_type {
                        if b_type == Some(TrackType::ListItems) {
                            println!("INTERRUPTING");
                            t.interrupt(b_type.unwrap(), true);
                            if let Some(j) = t.next_track_b() {
                                j.tag_b = b_attrs.tag();
                                j.is_original_b = true;
                            }
                            t.b_del.begin();
                        } else {
                            t.unenter_b();
                        }
                    } else {
                        t.enter_b(None, b_attrs.tag().unwrap());
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
                    t.skip_a(cmp::min(a_count, b_count));
                    t.skip_b(cmp::min(a_count, b_count));
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
                
                // With Groups
                (Some(AddWithGroup(a_inner)), Some(AddSkip(b_count))) => {
                    t.a_del.skip(1);
                    t.a_add.skip(1);
                    t.b_del.skip(1);
                    t.b_add.with_group(&a_inner);
                    
                    a.next();
                    if b_count > 1 {
                        b.head = Some(AddSkip(b_count - 1));
                    } else {
                        b.next();
                    }
                },
                (Some(AddWithGroup(a_inner)), Some(AddWithGroup(b_inner))) => {
                    let (a_op, b_op) = transform_insertions(&a_inner, &b_inner);
                    
                    t.a_del.with_group(&a_op.0);
                    t.a_add.with_group(&a_op.1);
                    t.b_del.with_group(&b_op.0);
                    t.b_add.with_group(&b_op.1);
                    
                    a.next();
                    b.next();
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

pub fn transform_deletions(avec: &DelSpan, bvec: &DelSpan) -> (DelSpan, DelSpan) {
    let mut a = DelStepper::new(avec);
    let mut b = DelStepper::new(bvec);

    let mut a_del = DelWriter::new();
    let mut b_del = DelWriter::new();

    while !(a.is_done() && b.is_done()) {
        println!("{}", Green.bold().paint("transform_deletions:"));
        println!("{}", BrightGreen.paint(format!(" @ a_del: {:?}", a_del)));
        println!("{}", BrightGreen.paint(format!(" @ b_del: {:?}", b_del)));

        if a.is_done() {
            println!("{}", BrightYellow.paint(format!("Finishing B: {:?}", b.head.clone())));

            match b.head.clone() {
                Some(DelGroup(ref span)) => {
                    // t.skip_b(1);
                    // t.group_a(attrs, span);
                    a_del.group(span);
                    b.next();
                },
                Some(DelWithGroup(ref span)) => {
                    a_del.with_group(span);
                    b.next();
                },
                Some(DelChars(b_chars)) => {
                    // t.chars_a(b_chars);
                    // t.skip_b(b_chars.len());
                    a_del.chars(b_chars);
                    b.next();
                },
                Some(DelSkip(b_count)) => {
                    // t.skip_a(b_count);
                    // t.skip_b(b_count);
                    a_del.skip(b_count);
                    b.next();
                },
                None => {
                    // t.close_b();
                    b.exit();
                },
                _ => {
                    panic!("What: {:?}", b.head);
                }
            }
        } else if b.is_done() {
            println!("{}", BrightYellow.paint(format!("Finishing A: {:?}", a.head.clone())));

            match a.head.clone() {
                Some(DelGroup(ref span)) => {
                    // t.skip_a(1);
                    // t.group_b(attrs, span);
                    a_del.skip(1);
                    b_del.group(span);
                    a.next();
                },
                Some(DelWithGroup(ref span)) => {
                    // t.skip_a(1);
                    // t.group_b(attrs, span);
                    b_del.with_group(span);
                    a_del.skip(1);
                    a.next();
                },
                Some(DelChars(ref a_chars)) => {
                    // t.skip_a(a_chars.len());
                    // t.chars_b(a_chars);
                    a.next();
                },
                Some(DelSkip(a_count)) => {
                    // t.skip_a(a_count);
                    // t.skip_b(a_count);
                    a.next();
                },
                None => {
                    // t.close_a();
                    a.exit();
                },
                _ => {
                    panic!("Unknown value: {:?}", a.head.clone());
                }
            }

        } else {
            println!("{}", BrightYellow.paint(format!("Next step: {:?}", (a.head.clone(), b.head.clone()))));

            match (a.head.clone(), b.head.clone()) {

                // Groups
                (Some(DelGroup(a_inner)), Some(DelGroup(b_inner))) => {
                    let (a_del_inner, b_del_inner) = transform_deletions(&a_inner, &b_inner);

                    a_del.place_all(&a_del_inner);
                    b_del.place_all(&b_del_inner);

                    a.next();
                    b.next();
                },

                // Rest
                (Some(DelSkip(a_count)), Some(DelSkip(b_count))) => {
                    if a_count > b_count {
                        a.head = Some(DelSkip(a_count - b_count));
                        b.next();
                    } else if a_count < b_count {
                        a.next();
                        b.head = Some(DelSkip(b_count - a_count));
                    } else {
                        a.next();
                        b.next();
                    }

                    a_del.skip(cmp::min(a_count, b_count));
                    b_del.skip(cmp::min(a_count, b_count));
                },
                (Some(DelSkip(a_count)), Some(DelChars(b_chars))) => {
                    b.next();
                    a_del.chars(b_chars);
                },
                (Some(DelChars(a_chars)), Some(DelChars(b_chars))) => {
                    if a_chars > b_chars {
                        a.head = Some(DelChars(a_chars - b_chars));
                        b.next();
                    } else if a_chars < b_chars {
                        a.next();
                        b.head = Some(DelChars(b_chars - a_chars));
                    } else {
                        a.next();
                        b.next();
                    }

                    a_del.skip(cmp::min(a_chars, b_chars));
                    b_del.skip(cmp::min(a_chars, b_chars));
                },
                (Some(DelChars(a_chars)), Some(DelSkip(b_count))) => {
                    if a_chars > b_count {
                        a.head = Some(DelChars(a_chars - b_count));
                        b.next();
                    } else if a_chars < b_count {
                        a.next();
                        b.head = Some(DelSkip(b_count - a_chars));
                    } else {
                        a.next();
                        b.next();
                    }

                    // a_del.skip(cmp::min(a_chars, b_chars));
                    b_del.chars(cmp::min(a_chars, b_count));
                },
                (Some(DelChars(a_chars)), _) => {
                    a.next();
                    b_del.chars(a_chars);
                },

                // With Groups
                (Some(DelWithGroup(a_inner)), Some(DelWithGroup(b_inner))) => {
                    let (a_del_inner, b_del_inner) = transform_deletions(&a_inner, &b_inner);

                    a_del.with_group(&a_del_inner);
                    b_del.with_group(&b_del_inner);

                    a.next();
                    b.next();
                },
                (Some(DelSkip(a_count)), Some(DelWithGroup(b_inner))) => {
                    a_del.with_group(&b_inner);
                    b_del.skip(1);

                    if a_count > 1 {
                        a.head = Some(DelSkip(a_count - 1));
                    } else {
                        a.next();
                    }
                    b.next();
                },
                (Some(DelWithGroup(a_inner)), Some(DelSkip(b_count))) => {
                    a_del.skip(1);
                    b_del.with_group(&a_inner);

                    a.next();
                    if b_count > 1 {
                        b.head = Some(DelSkip(b_count - 1));
                    } else {
                        b.next();
                    }
                },

                unimplemented => {
                    println!("Not reachable: {:?}", unimplemented);
                    unreachable!();
                }
            }
        }
    }

    let a_res = a_del.result();
    let b_res = b_del.result();

    println!("{}", BrightYellow.paint(format!("Result A: {:?}", a_res)));
    println!("{}", BrightYellow.paint(format!("Result B: {:?}", b_res)));

    (a_res, b_res)
}

/// Transforms a insertion preceding a deletion into a deletion preceding an insertion.
/// After this, sequential deletions and insertions can be composed together in one operation.

/*
function delAfterIns (insA, delB, schema) {
  var c, delr, insr, iterA, iterB, _ref;
  _ref = oatie.record(), delr = _ref[0], insr = _ref[1];
  iterA = new oatie.OpIterator(insA);
  iterB = new oatie.OpIterator(delB);
  iterA.next();
  iterB.next();
  c = oatie._combiner([delr, insr], iterA, true, iterB, false);
  c.delAfterIns = function () {
    var _ref1, _ref2, _ref3, _ref4, _ref5, _ref6, _ref7, _ref8, _ref9;
    while (!(iterA.isExit() || iterB.isExit())) {
      if ((_ref1 = iterA.type) === 'text') {
        c.useA();
        delr.retain();
      }
      if ((_ref2 = iterA.type) === 'open') {
        delr.enter(iterA.tag, iterA.attrs);
        c.useA();
      }
      if (iterA.type === 'retain' && (iterB.type === 'remove' || iterB.type === 'retain')) {
        c.pickB();
      } else if (iterA.type === 'retain' && (iterB.type === 'unopen' || iterB.type === 'enter')) {
        c.nextA().consumeB();
      } else if (((_ref6 = iterA.type) === 'enter' || _ref6 === 'attrs') && (iterB.type === 'remove')) {
        c.retainA();
      } else if (((_ref6 = iterA.type) === 'enter' || _ref6 === 'attrs') && (iterB.type === 'retain')) {
        c.retainA().nextB();
      } else if (((_ref8 = iterA.type) === 'enter' || _ref8 === 'attrs') && ((_ref9 = iterB.type) === 'enter' || _ref9 === 'unopen')) {
        c.pickB().delAfterIns().pickB();
      }
    }

    while (!iterA.isExit()) {
      c.retainA();
    }
    // Catch .close() ending.
    if (iterA.type == 'close') {
      delr.leave();
      c.useA();
      return this.delAfterIns();
    }
    while (!iterB.isExit()) {
      if (iterB.type === 'retain') {
        c.useB();
      } else {
        c.consumeB();
      }
    }
    return this;
  };
  return c.delAfterIns().toJSON();
}
*/

pub fn transform_add_del_inner(delres: &mut DelSpan, addres: &mut AddSpan, a: &mut AddSlice, b: &mut DelSlice) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            DelChars(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        addres.place(&AddChars(avalue.clone()));
                        delres.place(&DelSkip(avalue.len()));
                        a.next();
                    },
                    AddSkip(acount) => {
                        if bcount < acount {
                            a.head = Some(AddSkip(acount - bcount));
                            delres.place(&b.next());
                        } else if bcount > acount {
                            a.next();
                            delres.place(&DelChars(acount));
                            b.head = Some(DelChars(bcount - acount));
                        } else {
                            a.next();
                            delres.place(&b.next());
                        }
                    },
                    _ => {
                        panic!("Unimplemented or Unexpected");
                    },
                }
            },
            DelSkip(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        addres.place(&AddChars(avalue.clone()));
                        delres.place(&DelSkip(avalue.len()));
                        a.next();
                    },
                    AddSkip(acount) => {
                        addres.place(&AddSkip(cmp::min(acount, bcount)));
                        delres.place(&DelSkip(cmp::min(acount, bcount)));
                        if acount > bcount {
                            a.head = Some(AddSkip(acount - bcount));
                            b.next();
                        } else if acount < bcount {
                            a.next();
                            b.head = Some(DelSkip(bcount - acount));
                        } else {
                            a.next();
                            b.next();
                        }
                    },
                    AddWithGroup(..) => {
                        addres.place(&a.next());
                        delres.place(&DelSkip(1));
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(DelSkip(bcount - 1));
                        }
                    },
                    AddGroup(tags, span) => {
                        let mut a_inner = AddSlice::new(&span);
                        let mut delres_inner: DelSpan = vec![];
                        let mut addres_inner: AddSpan = vec![];
                        transform_add_del_inner(&mut delres_inner, &mut addres_inner, &mut a_inner, b);
                        if !a_inner.is_done() {
                            addres_inner.place(&a_inner.head.unwrap());
                            addres_inner.place_all(a_inner.rest);
                        }
                        addres.place(&AddGroup(tags, addres_inner));
                        delres.place(&DelWithGroup(delres_inner));
                        a.next();
                    },
                }
            },
            DelWithGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelWithGroup by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        addres.place(&AddSkip(1));
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = transform_add_del(&insspan, &span);
                        delres.place(&DelWithGroup(del));
                        addres.place(&AddWithGroup(ins));
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();

                        let (_, ins) = transform_add_del(&insspan, &span);
                        addres.place(&AddGroup(attr, ins));
                    },
                }
            },
            DelGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelGroup by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        addres.place(&AddSkip(1));
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = transform_add_del(&insspan, &span);
                        delres.place(&DelGroup(del));
                        addres.place_all(&ins[..]);
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = transform_add_del(&insspan, &span);
                        delres.place_all(&del[..]);
                        addres.place_all(&ins[..]);
                    },
                }
            },
            DelGroupAll => {
                match a.get_head() {
                    AddChars(avalue) => {
                        panic!("DelGroupAll by AddChars is ILLEGAL");
                    },
                    AddSkip(acount) => {
                        delres.place(&b.next());
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    },
                    AddWithGroup(insspan) => {
                        a.next();
                        delres.place(&b.next());
                    },
                    AddGroup(attr, insspan) => {
                        a.next();
                        b.next();
                    },
                }
            },
        }
    }
}


pub fn transform_add_del(avec: &AddSpan, bvec: &DelSpan) -> Op {
    let mut delres: DelSpan = Vec::with_capacity(avec.len() + bvec.len());
    let mut addres: AddSpan = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddSlice::new(avec);
    let mut b = DelSlice::new(bvec);

    transform_add_del_inner(&mut delres, &mut addres, &mut a, &mut b);

    if !b.is_done() {
        delres.place_all(&b.into_span());
    }

    if !a.is_done() {
        let rest = a.into_span();
        delres.place(&DelSkip(rest.skip_len()));
        addres.place_all(&rest);
    }

    (delres, addres)
}

/// Transform two operations according to a schema.

pub fn transform(a: &Op, b: &Op) -> (Op, Op) {
    // Transform deletions A and B against each other to get delA` and delB`.
    println!(" # transform[1] transform_deletions");
    println!(" a_del   {:?}", a.0);
    println!(" b_del   {:?}", b.0);
    let (a_del_0, b_del_0) = transform_deletions(&a.0, &b.0);
    println!(" == a_del_0 {:?}", a_del_0);
    println!(" == b_del_0 {:?}", b_del_0);
    println!("");

    // The result will be applied after the client's insert operations had already been performed.
    // Reverse the impact of insA with delA` to not affect already newly added elements or text.
    // var _ = delAfterIns(insA, delA_0, schema), delA_1 = _[0];
    // var _ = delAfterIns(insB, delB_0), delB_1 = _[0];
    println!(" # transform[2] transform_add_del");
    println!(" a_ins   {:?}", a.1);
    println!(" a_del_0 {:?}", a_del_0);
    println!(" ~ transform_add_del()");
    let (a_del_1, a_ins_1) = transform_add_del(&a.1, &a_del_0);
    println!(" == a_del_1 {:?}", a_del_1);
    println!(" == a_ins_1 {:?}", a_ins_1);
    println!("");

    println!(" # transform[3] transform_add_del");
    println!(" b_ins   {:?}", b.1);
    println!(" b_del_0 {:?}", b_del_0);
    println!(" ~ transform_add_del()");
    let (b_del_1, b_ins_1) = transform_add_del(&b.1, &b_del_0);
    println!(" == b_del_1 {:?}", b_del_1);
    println!(" == b_ins_1 {:?}", b_ins_1);
    println!("");

    // Insertions from both clients must be composed as though they happened against delA` and delB`
    // so that we don't have phantom elements.
    //var _ = oatie._composer(insA, true, delA_1, false).compose().toJSON(), insA1 = _[1];
    // var _ = oatie._composer(insB, true, delB_1, false).compose().toJSON(), insB1 = _[1];
    // let a_ins_1 = a.clone().1;
    // let b_ins_1 = b.clone().1;

    // Transform insert operations together.
    println!(" # transform[4] transform_insertions");
    println!(" a_ins_1 {:?}", a_ins_1);
    println!(" b_ins_1 {:?}", b_ins_1);
    let ((a_del_2, a_ins_2), (b_del_2, b_ins_2)) = transform_insertions(&a_ins_1, &b_ins_1);
    println!(" == a_del_2 {:?}", a_del_2);
    println!(" == a_ins_2 {:?}", a_ins_2); // == a_ins_2 [AddWithGroup([AddWithGroup([AddWithGroup([AddSkip(8)]), AddChars("a")])])]
    println!(" == b_del_2 {:?}", b_del_2);
    println!(" == b_ins_2 {:?}", b_ins_2);
    println!("");

    // Our delete operations are now subsequent operations, and so can be composed.
    //var _ = oatie._composer(delA_1, false, delA_2, false).compose().toJSON(), delA_3 = _[0], _ = _[1];
    //var _ = oatie._composer(delB_1, false, delB_2, false).compose().toJSON(), delB_3 = _[0], _ = _[1];
    println!(" # transform[5] compose_del_del");
    println!(" a_del_1 {:?}", a_del_1);
    println!(" a_del_2 {:?}", a_del_2);
    let a_del_3 = compose::compose_del_del(&a_del_1, &a_del_2);
    println!(" == a_del_3 {:?}", a_del_3);
    println!("");
    println!(" # transform[6] compose_del_del");
    println!(" b_del_1 {:?}", b_del_1);
    println!(" b_del_2 {:?}", b_del_2);
    let b_del_3 = compose::compose_del_del(&b_del_1, &b_del_2);
    println!(" == b_del_3 {:?}", b_del_3);
    println!("");

    println!(" # transform[result]");
    println!(" a_del   {:?}", a.0);
    println!(" a_ins   {:?}", a.1);
    println!(" ~ transform()");
    println!(" =a_del_3  {:?}", a_del_3);
    println!(" =a_ins_2  {:?}", a_ins_2);
    println!(" ---");
    println!(" b_del   {:?}", b.0);
    println!(" b_ins   {:?}", b.1);
    println!(" ~ transform()");
    println!(" =b_del_3  {:?}", b_del_3); // wrong
    println!(" =b_ins_2  {:?}", b_ins_2);
    println!("");

    ((a_del_3, a_ins_2), (b_del_3, b_ins_2))
}
