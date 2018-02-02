//! Performs operational transform.

use std::collections::HashMap;
use std::borrow::ToOwned;
use std::cmp;

use super::doc::*;
use super::stepper::*;
use super::compose;
use super::normalize;
use super::writer::*;

use term_painter::ToStyle;
use term_painter::Color::*;
use term_painter::Attr::*;
use std::collections::HashSet;
use std::marker::PhantomData;
use std::fmt::Debug;

pub trait Track: Copy + Debug + PartialEq + Sized {
    // Rename this do close split? if applicable?
    fn do_split(&self) -> bool;

    // Unsure about this naming
    fn do_open_split(&self) -> bool;

    fn supports_text(&self) -> bool;

    fn allowed_in_root(&self) -> bool;

    // TODO is this how this should work
    fn is_object(&self) -> bool;

    fn parents(&self) -> Vec<Self>;

    // TODO extrapolate this from parents()
    fn ancestors(&self) -> Vec<Self>;
}

pub trait Schema: Clone + Debug {
    type Track: Track + Sized;

    /// Determines if two sets of Attrs are equal.
    fn attrs_eq(a: &Attrs, b: &Attrs) -> bool;

    /// Get the track type from this Attrs.
    fn track_type_from_attrs(attrs: &Attrs) -> Option<Self::Track>;

    /// Combine two Attrs into a new definition.
    fn merge_attrs(a: &Attrs, b: &Attrs) -> Option<Attrs>;
}


#[derive(Clone, Debug)]
struct TrackState<S> where S: Schema {
    tag_a: Option<Attrs>,
    tag_real: Option<Attrs>,
    tag_b: Option<Attrs>,
    is_original_a: bool,
    is_original_b: bool,
    _phantom: PhantomData<S>,
}

struct Transform<S> where S: Schema {
    tracks: Vec<TrackState<S>>,
    a_del: DelWriter,
    a_add: AddWriter,
    b_del: DelWriter,
    b_add: AddWriter,
}

impl<S> Transform<S> where S: Schema {
    fn new() -> Transform<S> {
        Transform {
            tracks: vec![],
            a_del: DelWriter::new(),
            a_add: AddWriter::new(),
            b_del: DelWriter::new(),
            b_add: AddWriter::new(),
        }
    }

    fn enter(&mut self, name: &Attrs) {
        let last = self.tracks
            .iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

        self.tracks.insert(
            last,
            TrackState {
                tag_a: Some(name.clone()),
                tag_real: Some(name.clone()),
                tag_b: Some(name.clone()),
                is_original_a: true,
                is_original_b: true,
                _phantom: PhantomData,
            },
        );

        self.a_del.begin();
        self.a_add.begin();
        self.b_del.begin();
        self.b_add.begin();
    }

    // TODO maybe take "real" value as input?
    fn enter_a(&mut self, a: &Attrs, b: Option<Attrs>) {
        let last = self.tracks
            .iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

        // TODO merge this functionality elsewhere
        let real = if let Some(ref b) = b {
            Some(S::merge_attrs(a, b).unwrap_or_else(|| a.clone()))
        } else {
            Some(a.clone())
        };

        // if let Some(last) = self.tracks.last() {
        //     if a.tag_type() == last._anything().unwrap().tag_type() {
        //         log_transform!("-----> UGH {:?}", last);
        //         panic!("Should not have consecutive similar tracks.");
        //     }
        // }

        self.tracks.insert(
            last,
            TrackState {
                tag_a: Some(a.clone()),
                tag_real: real,
                tag_b: b.clone(),
                is_original_a: true,
                is_original_b: false,
                _phantom: PhantomData,
            },
        );

        self.a_del.begin();
        self.a_add.begin();
        if b.is_some() {
            self.b_del.begin();
        }
        self.b_add.begin();
    }

    fn enter_b(&mut self, a: Option<Attrs>, b: &Attrs) {
        log_transform!("ENTER B");

        let last = self.tracks
            .iter()
            .rposition(|x| x.tag_real.is_some())
            .and_then(|x| Some(x + 1))
            .unwrap_or(0);

        // TODO merge this functionality elsewhere
        let real = if let Some(ref a) = a {
            Some(S::merge_attrs(a, &b).unwrap_or_else(|| b.clone()))
        } else {
            Some(b.clone())
        };

        log_transform!("dump all {:?}", self.tracks);

        if let Some(last) = self.tracks.last() {
            if S::track_type_from_attrs(b) == S::track_type_from_attrs(last.tag_a.as_ref().or(last.tag_real.as_ref()).or(last.tag_b.as_ref()).unwrap()) {
                log_transform!("-----> UGH {:?}", last);
                panic!("Should not have consecutive similar tracks.");
            }
        }

        self.tracks.insert(
            last,
            TrackState {
                tag_a: a.clone(),
                tag_real: real,
                tag_b: Some(b.clone()),
                is_original_a: false,
                is_original_b: true,
                _phantom: PhantomData,
            },
        );

        if a.is_some() {
            self.a_del.begin();
        }
        self.a_add.begin();
        self.b_del.begin();
        self.b_add.begin();
    }

    // Close the topmost track.
    fn abort(&mut self) -> (Option<Attrs>, Option<Attrs>, Option<Attrs>) {
        let track = self.tracks.pop().unwrap();

        if let Some(ref real) = track.tag_real {
            self.a_add.close(real.clone());
            self.b_add.close(real.clone());
        }

        (track.tag_a, track.tag_real, track.tag_b)
    }

    fn unenter_a(&mut self, ty: S::Track) {
        self.a_del.begin();
        let track = self.next_track_a_by_type(ty).unwrap();
        track.tag_a = track.tag_real.clone();
    }

    fn unenter_b(&mut self, ty: S::Track) {
        self.b_del.begin();
        let track = self.next_track_b_by_type(ty).unwrap();
        track.tag_b = track.tag_real.clone();
    }

    fn skip_a(&mut self, n: usize) {
        self.a_del.place(&DelSkip(n));
        self.a_add.place(&AddSkip(n));
    }

    fn skip_b(&mut self, n: usize) {
        self.b_del.place(&DelSkip(n));
        self.b_add.place(&AddSkip(n));
    }

    fn with_group_a(&mut self, span: &AddSpan) {
        self.a_add.place(&AddWithGroup(span.clone()));
    }

    fn with_group_b(&mut self, span: &AddSpan) {
        self.b_add.place(&AddWithGroup(span.clone()));
    }

    fn group_a(&mut self, attrs: &Attrs, span: &AddSpan) {
        self.a_add.place(&AddGroup(attrs.clone(), span.clone()));
    }

    fn group_b(&mut self, attrs: &Attrs, span: &AddSpan) {
        self.b_add.place(&AddGroup(attrs.clone(), span.clone()));
    }

    fn chars_a(&mut self, chars: &str) {
        self.a_add.place(&AddChars(chars.to_owned()));
    }

    fn chars_b(&mut self, chars: &str) {
        self.b_add.place(&AddChars(chars.to_owned()));
    }

    fn current(&self) -> Option<TrackState<S>> {
        let value = self.tracks.last();
        if let Some(track) = value {
            Some((*track).clone())
        } else {
            None
        }
    }

    fn close(&mut self) {
        let (track, index) = self.top_track_a().unwrap();

        if track.is_original_a && track.tag_real == track.tag_a {
            self.a_del.exit();
            self.a_add.exit();
        } else {
            self.a_del.close();
            self.a_add.close(track.tag_real.clone().unwrap());
        }

        if track.is_original_b && track.tag_real == track.tag_b {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            self.b_del.close();
            self.b_add.close(track.tag_real.clone().unwrap());
        }

        self.tracks.remove(index);
    }

    fn top_track_a(&mut self) -> Option<(TrackState<S>, usize)> {
        self.tracks
            .iter()
            .rposition(|x| x.tag_a.is_some())
            .map(|index| (self.tracks[index].clone(), index))
    }

    fn top_track_real(&self) -> Option<(TrackState<S>, usize)> {
        self.tracks
            .iter()
            .rposition(|x| x.tag_real.is_some())
            .map(|index| (self.tracks[index].clone(), index))
    }

    fn top_track_b(&mut self) -> Option<(TrackState<S>, usize)> {
        self.tracks.iter()
            .rposition(|x| x.tag_b.is_some())
            .map(|index| (self.tracks[index].clone(), index))
    }

    fn next_track_a_by_type(&mut self, arg: S::Track) -> Option<&mut TrackState<S>> {
        if let Some(track) = self.tracks.iter()
            .position(|x| x.tag_a.is_none() && x.tag_real.as_ref().and_then(|x| S::track_type_from_attrs(x)) == Some(arg)) {
            Some(&mut self.tracks[track])
        } else {
            None
        }
    }

    fn next_track_b_by_type(&mut self, arg: S::Track) -> Option<&mut TrackState<S>> {
        if let Some(track) = self.tracks.iter()
            .position(|x| x.tag_b.is_none() && x.tag_real.as_ref().and_then(|x| S::track_type_from_attrs(x)) == Some(arg)) {
            Some(&mut self.tracks[track])
        } else {
            None
        }
    }

    fn close_a(&mut self) {
        let (track, index) = self.top_track_a().unwrap();

        // Determine whether to split tags for this track type.
        // TODO do the same for track opening?
        // TODO remove do_split checks from this class? no-schema knowledge possible?
        let track_split = if let Some(tag) = track.tag_real.clone() {
            S::track_type_from_attrs(&tag).map_or(false, |x| x.do_split())
        } else {
            true
        };

        if track.is_original_a && (track_split || track.tag_b.is_none()) {
            self.a_del.exit();
            self.a_add.exit();
        } else {
            self.a_del.close();
            if track_split || track.tag_b.is_none() {
                self.a_add.close(track.tag_real.clone().unwrap());
            }
        }

        if track_split || track.tag_b.is_none() {
            self.b_add.close(track.tag_real.clone().unwrap());
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
    }

    fn close_b(&mut self) {
        let (track, index) = self.top_track_b().unwrap();

        // Determine whether to split tags for this track type.
        // TODO do the same for track opening?
        // TODO remove do_split checks from this class? no-schema knowledge possible?
        let track_split = if let Some(ref tag) = track.tag_real.clone() {
            S::track_type_from_attrs(tag).map_or(false, |x| x.do_split())
        } else {
            true
        };

        if track.is_original_b && (track_split || track.tag_a.is_none()) {
            self.b_del.exit();
            self.b_add.exit();
        } else {
            self.b_del.close();
            if track_split || track.tag_a.is_none() {
                self.b_add.close(track.tag_real.clone().unwrap());
            }
        }

        if track_split || track.tag_a.is_none() {
            self.a_add.close(track.tag_real.clone().unwrap());
        }

        if track.tag_a.is_none() {
            self.tracks.remove(index);
        } else {
            if track_split {
                self.tracks[index].is_original_a = false;
            }
            if track_split {
                self.tracks[index].is_original_b = false;
            }
            self.tracks[index].tag_b = None;
            if track_split {
                self.tracks[index].tag_real = None;
            }
        }
    }

    // Interrupt all tracks up the ancestry until we get to
    // a particular type, OR a type than could be an ancestor
    // of the given type
    fn interrupt(&mut self, itype: S::Track, inclusive: bool) {
        let mut regen = vec![];
        while let Some(track) = self.current() {
            let (istag, hasparent) = if let Some(ref real) = track.tag_real {
                log_transform!("WOW {:?} {:?}", real, itype);
                let tag_type = S::track_type_from_attrs(real).unwrap();
                (
                    tag_type == itype,
                    tag_type.ancestors().iter().any(|x| *x == itype),
                )
            } else {
                (false, false)
            };

            if track.tag_real.is_some() && ((!istag && hasparent) || (istag && inclusive)) {
                log_transform!("aborting by {:?} {:?} {:?}", itype, inclusive, istag);
                let aborted = self.abort();
                regen.push(aborted);
                if istag && inclusive {
                    break;
                }
            } else {
                break;
            }
        }

        for group in regen {
            self.tracks.push(TrackState {
                tag_a: group.0,
                tag_real: None,
                tag_b: group.2,
                is_original_a: false,
                is_original_b: false,
                _phantom: PhantomData,
            })
        }
    }

    // TODO combine this with regenerate_until ?
    fn regenerate(&mut self) {
        // Filter for types that are ancestors of the current type.
        for track in &mut self.tracks {
            if track.tag_real.is_none() {
                log_transform!("REGENERATE: {:?}", track);
                if let (&Some(ref a), &Some(ref b)) = (&track.tag_a, &track.tag_b) {
                    track.tag_real = Some(S::merge_attrs(a, b).unwrap_or_else(|| a.clone()));
                    track.is_original_a = false;
                    track.is_original_b = false;
                } else if track.tag_b.is_some() {
                    track.tag_real = track.tag_b.clone();
                } else if track.tag_a.is_some() {
                    track.tag_real = track.tag_a.clone();
                }

                self.a_add.begin();
                self.b_add.begin();
            }
        }
    }

    fn regenerate_until(&mut self, target: S::Track) {
        // Filter for types that are ancestors of the current type.
        for track in &mut self.tracks {
            if track.tag_real.is_none() {
                log_transform!("REGENERATE UNTIL: {:?}", target);

                // Get the type of this track.
                // TODO is there a better way of doing this? Store track type in the track?
                let track_type = track.tag_a.as_ref().map(|t| S::track_type_from_attrs(t).unwrap())
                    .or(track.tag_real.as_ref().map(|t| S::track_type_from_attrs(t).unwrap()))
                    .or(track.tag_b.as_ref().map(|t| S::track_type_from_attrs(t).unwrap()))
                    .unwrap();

                if target.ancestors().iter().position(|x| *x == track_type).is_none()
                    && target != track_type {
                    if target == track_type {
                        log_transform!("met {:?}", target);
                        break;
                    } else {
                        log_transform!(":O mismatched ancestor {:?} of {:?}", track_type, target);
                        break;
                    }
                } else {
                    log_transform!(":) regen {:?}", track_type);
                }

                if let (&Some(ref a), &Some(ref b)) = (&track.tag_a, &track.tag_b) {
                    track.tag_real = Some(S::merge_attrs(a, b).unwrap_or_else(|| a.clone()));
                    track.is_original_a = false;
                    track.is_original_b = false;
                } else if track.tag_b.is_some() {
                    track.tag_real = track.tag_b.clone();
                } else if track.tag_a.is_some() {
                    track.tag_real = track.tag_a.clone();
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
            log_transform!("TRACK RESULT: {:?}", track);
            if !track.is_original_a && track.tag_real.is_some() {
                a_add.close(track.tag_a.clone().unwrap());
            }
            if track.is_original_a {
                a_del.exit();
                a_add.exit();
            }
            if !track.is_original_b && track.tag_real.is_some() {
                b_add.close(track.tag_b.clone().unwrap());
            }
            if track.is_original_b {
                b_del.exit();
                b_add.exit();
            }
        }
        (
            (a_del.result(), a_add.result()),
            (b_del.result(), b_add.result()),
        )
    }

    fn current_type(&self) -> Option<S::Track> {
        S::track_type_from_attrs(self.tracks
            .last()
            .unwrap()
            .tag_real
            .as_ref()
            .unwrap())
    }

    fn supports_text(&self) -> bool {
        self.top_track_real()
            .map(|(track, _)| S::track_type_from_attrs(track.tag_real.as_ref().unwrap()).unwrap().supports_text())
            .unwrap_or(false)
    }
}

pub fn transform_insertions<S: Schema>(avec: &AddSpan, bvec: &AddSpan) -> (Op, Op) {
    let mut a = AddStepper::new(avec);
    let mut b = AddStepper::new(bvec);

    let mut t: Transform<S> = Transform::new();

    while !(a.is_done() && b.is_done()) {
        log_transform!("{}", Green.bold().paint("Tracks:"));
        for t in &t.tracks {
            log_transform!("{}", BrightGreen.paint(format!(" - {:?}", t)));
        }

        log_transform!("{}", BrightGreen.paint(format!(" @ a_del: {:?}", t.a_del)));
        log_transform!("{}", BrightGreen.paint(format!(" @ a_add: {:?}", t.a_add)));
        log_transform!("{}", BrightGreen.paint(format!(" @ b_del: {:?}", t.b_del)));
        log_transform!("{}", BrightGreen.paint(format!(" @ b_add: {:?}", t.b_add)));

        if a.is_done() {
            // log_transform!("tracks {:?}", t.tracks);
            t.regenerate();
            log_transform!(
                "{}",
                BrightYellow.paint(format!("Finishing B (ins): {:?}", b.head.clone()))
            );

            match b.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_b(1);
                    t.group_a(attrs, span);
                    b.next();
                }
                Some(AddWithGroup(ref span)) => {
                    t.skip_b(1);
                    t.with_group_a(span);
                    b.next();
                }
                Some(AddChars(ref b_chars)) => {
                    t.chars_a(b_chars);
                    t.skip_b(b_chars.len());
                    b.next();
                }
                Some(AddSkip(b_count)) => {
                    t.skip_a(b_count);
                    t.skip_b(b_count);
                    b.next();
                }
                None => {
                    t.close_b();
                    b.exit();
                }
            }
        } else if b.is_done() {
            t.regenerate();
            log_transform!(
                "{}",
                BrightYellow.paint(format!("Finishing A (add): {:?}", a.head.clone()))
            );

            match a.head.clone() {
                Some(AddGroup(ref attrs, ref span)) => {
                    t.skip_a(1);
                    t.group_b(attrs, span);
                    a.next();
                }
                Some(AddWithGroup(ref span)) => {
                    t.skip_a(1);
                    t.with_group_b(span);
                    a.next();
                }
                Some(AddChars(ref a_chars)) => {
                    t.skip_a(a_chars.len());
                    t.chars_b(a_chars);
                    a.next();
                }
                Some(AddSkip(a_count)) => {
                    t.skip_a(a_count);
                    t.skip_b(a_count);
                    a.next();
                }
                None => {
                    t.close_a();
                    a.exit();
                }
            }
        } else {
            log_transform!(
                "{}",
                BrightYellow.paint(format!("Next step (ins):\n A {:?}\n B {:?}", a.head.clone(), b.head.clone()))
            );

            match (a.head.clone(), b.head.clone()) {
                // Closing
                (None, None) => {
                    let (a_tag, b_tag) = {
                        let t = t.tracks.last().unwrap();
                        (t.tag_a.clone(), t.tag_b.clone())
                    };

                    if a_tag.is_some() && b_tag.is_some() &&
                        S::track_type_from_attrs(a_tag.as_ref().unwrap()) == S::track_type_from_attrs(b_tag.as_ref().unwrap())
                    {
                        // t.interrupt(a_tag || b_tag);
                        a.exit();
                        b.exit();
                        t.close();
                    } else if a_tag.is_some() &&
                               (b_tag.is_none() ||
                                    S::track_type_from_attrs(a_tag
                                        .as_ref()
                                        .unwrap())
                                        .unwrap()
                                        .ancestors()
                                        .iter()
                                        .any(|x| *x == S::track_type_from_attrs(b_tag.as_ref().unwrap()).unwrap()))
                    {
                        // t.interrupt(a_tag);
                        a.exit();
                        t.close_a();
                    } else if b_tag.is_some() {
                        // t.interrupt(b_tag);
                        b.exit();
                        t.close_b();
                    }
                }

                // TODO don't like that this isn't a pattern match;
                // This case should handle AddWithGroup and AddGroup (I believe)
                (None, compare) => {
                    let ok = if let Some(AddChars(ref b_chars)) = compare {
                        if t.supports_text() {
                            t.chars_a(b_chars);
                            t.skip_b(b_chars.chars().count());
                            b.next();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // TODO this logic is evidence AddObject should be broken out
                    let groupsuccess = if let Some(AddGroup(ref b_attrs, _)) = compare {
                        let b_type = S::track_type_from_attrs(&b_attrs).unwrap();

                        if b_type.is_object() {
                            b.enter();
                            b.exit();
                            t.enter_b(None, b_attrs);
                            t.close_b();

                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !ok && !groupsuccess {
                        let a_typ = S::track_type_from_attrs(t.tracks
                            .iter()
                            .rev()
                            .find(|t| t.tag_a.is_some())
                            .unwrap()
                            .tag_a
                            .as_ref()
                            .unwrap())
                            .unwrap();
                        log_transform!("what is up with a {:?}", t.a_add);
                        t.interrupt(a_typ, false);
                        // log_transform!("... {:?} {:?}", t.a_del, t.a_add);
                        // log_transform!("... {:?} {:?}", t.b_del, t.b_add);
                        log_transform!("~~~> tracks {:?}", t.tracks);
                        t.close_a();
                        // log_transform!("...");
                        a.exit();
                        log_transform!("<~~~ tracks {:?}", t.tracks);
                        // log_transform!("WHERE ARE WE WITH A {:?}", a);
                    }
                }

                // Opening
                (Some(AddGroup(ref a_attrs, _)), Some(AddGroup(ref b_attrs, _))) => {
                    let a_type = S::track_type_from_attrs(a_attrs).unwrap();
                    let b_type = S::track_type_from_attrs(b_attrs).unwrap();

                    let b_is_child_of_a =
                        S::track_type_from_attrs(b_attrs)
                            .unwrap()
                            .ancestors()
                            .iter()
                            .any(|x| *x == S::track_type_from_attrs(a_attrs).unwrap());

                    log_transform!("GroupByGroup {:?} {:?}", a_type, b_type);

                    if a_type == b_type {
                        t.regenerate_until(a_type);

                        if a_type.is_object() && b_type.is_object() {
                            a.enter();
                            a.exit();
                            b.enter();
                            b.exit();
                            t.enter_a(&a_attrs, None);
                            t.close_a();
                            t.enter_b(None, &b_attrs);
                            t.close_b();
                        } else {
                            a.enter();
                            b.enter();
                            if S::attrs_eq(a_attrs, b_attrs) {
                                t.enter(&a_attrs);
                            } else {
                                t.enter_a(&a_attrs, Some(b_attrs.clone()));
                            }
                        }
                    } else if b_is_child_of_a {
                        t.regenerate_until(a_type);

                        a.enter();

                        log_transform!("~~~~ :O");
                        log_transform!("~~~~ -> {:?} {:?}", t.next_track_a_by_type(a_type), a_type);
                        if t.next_track_a_by_type(a_type).is_some() {
                            // if a_type.map_or(false, |x| x.do_open_split()) {
                            if true {
                                log_transform!("INTERRUPTING A");
                                t.interrupt(a_type.clone(), false);
                                log_transform!("BUT THE TRACKS -----<> {:?}", t.tracks);
                                if let Some(j) = t.next_track_a_by_type(a_type) {
                                    j.tag_a = Some(a_attrs.clone());
                                    j.is_original_a = false;
                                    log_transform!("inject A");
                                }
                                t.a_del.begin();
                            } else {
                                // TODO t.interrupt(a_type, true);
                                t.unenter_a(a_type);
                            }
                        } else {
                            t.interrupt(a_type.clone(), false); // caret-46
                            t.enter_a(&a_attrs, None);
                        }
                    } else /* a is a child of b */ {
                        t.regenerate_until(b_type);

                        b.enter();

                        // log_transform!("TELL ME {:?} {:?}", t.next_track_by_type(b_type.unwrap()), b_type);

                        if t.next_track_b_by_type(b_type.clone()).is_some() {
                            // if b_type.map_or(false, |x| x.do_open_split()) {
                            if true {
                                log_transform!("INTERRUPTING B");
                                t.interrupt(b_type.clone(), false);
                                if let Some(j) = t.next_track_b_by_type(b_type.clone()) {
                                    j.tag_b = Some(b_attrs.clone());
                                    j.is_original_b = false;
                                    log_transform!("inject B");
                                }
                                t.b_del.begin();
                            } else {
                                // TODO t.interrupt(b_type, true);
                                t.unenter_b(b_type);
                            }
                        } else {
                            t.interrupt(b_type.clone(), false); // caret-43
                            t.enter_b(None, &b_attrs);
                        }
                    }

                    // TODO if they are different tags THEN WHAT
                }
                (compare, None) => {
                    let is_char = if let Some(AddChars(a_chars)) = compare.clone() {
                        if t.supports_text() {
                            t.regenerate();

                            t.skip_a(a_chars.len());
                            t.chars_b(&a_chars);
                            a.next();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if !is_char {
                        // TODO this logic is evidence AddObject should be broken out
                        let groupsuccess = if let Some(AddGroup(ref a_attrs, _)) = compare {
                            t.regenerate_until(S::track_type_from_attrs(a_attrs).unwrap());
                            if S::track_type_from_attrs(a_attrs).unwrap().is_object() {
                                a.enter();
                                a.exit();
                                t.enter_a(a_attrs, None);
                                t.close_a();

                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        if !groupsuccess {
                            let b_typ = S::track_type_from_attrs(t.tracks
                                .iter()
                                .rev()
                                .find(|t| t.tag_b.is_some())
                                .unwrap()
                                .tag_b
                                .as_ref()
                                .unwrap())
                                .unwrap();
                            t.interrupt(b_typ, false);
                            t.close_b();
                            b.exit();
                        }
                    }
                }
                (Some(AddGroup(ref a_attrs, _)), _) => {
                    let a_type = S::track_type_from_attrs(a_attrs).unwrap();
                    t.regenerate_until(a_type);

                    // TODO should carets be worked around like this?
                    if a_type.is_object() {
                        // Carets
                        a.enter();
                        a.exit();
                        t.enter_a(&a_attrs, None);
                        t.close_a();
                    } else {
                        a.enter();

                        log_transform!("~~~~ :) :) :)");
                        log_transform!("~~~~ -> {:?} {:?}", t.next_track_a_by_type(a_type), a_type);
                        // in/15, caret-34
                        // if t.next_track_a_type() == a_type {
                        if t.next_track_a_by_type(a_type).is_some() {
                            if a_type.do_open_split() {
                            // if true {
                                log_transform!("INTERRUPTING A");
                                t.interrupt(a_type, true);
                                if let Some(j) = t.next_track_a_by_type(a_type) {
                                    j.tag_a = Some(a_attrs.clone());
                                    j.is_original_a = true;
                                }
                                t.a_del.begin();
                            } else {
                                t.unenter_a(a_type);
                            }
                        } else {
                            t.interrupt(a_type, true);
                            t.enter_a(&a_attrs, None);
                        }
                    }
                }
                (_, Some(AddGroup(ref b_attrs, _))) => {
                    let b_type = S::track_type_from_attrs(b_attrs).unwrap();
                    t.regenerate_until(b_type);

                    if b_type.is_object() {
                        b.enter();
                        b.exit();
                        t.enter_b(None, &b_attrs);
                        t.close_b();
                    } else {
                        // log_transform!("groupgruop {:?} {:?}", a_type, b_type);
                        b.enter();
                        // let b_type = Tag::from_attrs(b_attrs).tag_type();

                        if t.next_track_b_by_type(b_type).is_some() {
                            if b_type.do_open_split() {
                                log_transform!("INTERRUPTING B");
                                t.interrupt(b_type, true);
                                if let Some(j) = t.next_track_b_by_type(b_type) {
                                    j.tag_b = Some(b_attrs.clone());
                                    j.is_original_b = true;
                                }
                                t.b_del.begin();
                            } else {
                                // TODO? t.interrupt(b_type, true);
                                t.unenter_b(b_type);
                            }
                        } else {
                            t.interrupt(b_type, false); // caret-32
                            t.enter_b(None, &b_attrs);
                        }
                    }
                }

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
                }
                (Some(AddSkip(a_count)), Some(AddChars(ref b_chars))) => {
                    t.regenerate();

                    b.next();
                    t.chars_a(b_chars);
                    t.skip_b(b_chars.chars().count());
                }
                (Some(AddChars(ref a_chars)), _) => {
                    t.regenerate();

                    t.skip_a(a_chars.chars().count());
                    t.chars_b(a_chars);
                    a.next();
                }

                // With Groups
                (Some(AddWithGroup(a_inner)), Some(AddSkip(b_count))) => {
                    t.regenerate(); // caret-31

                    t.a_del.place(&DelSkip(1));
                    t.a_add.place(&AddSkip(1));
                    t.b_del.place(&DelSkip(1));
                    t.b_add.place(&AddWithGroup(a_inner));

                    a.next();
                    if b_count > 1 {
                        b.head = Some(AddSkip(b_count - 1));
                    } else {
                        b.next();
                    }
                }
                (Some(AddWithGroup(a_inner)), Some(AddWithGroup(b_inner))) => {
                    t.regenerate(); // caret-31

                    let (a_op, b_op) = transform_insertions::<S>(&a_inner, &b_inner);

                    t.a_del.place(&DelWithGroup(a_op.0));
                    t.a_add.place(&AddWithGroup(a_op.1));
                    t.b_del.place(&DelWithGroup(b_op.0));
                    t.b_add.place(&AddWithGroup(b_op.1));

                    a.next();
                    b.next();
                }
                (Some(AddSkip(a_count)), Some(AddWithGroup(b_inner))) => {
                    t.regenerate(); // caret-31

                    t.a_del.place(&DelSkip(1));
                    t.a_add.place(&AddWithGroup(b_inner));
                    t.b_del.place(&DelSkip(1));
                    t.b_add.place(&AddSkip(1));

                    if a_count > 1 {
                        a.head = Some(AddSkip(a_count - 1));
                    } else {
                        a.next();
                    }
                    b.next();
                }
                (Some(AddWithGroup(ref a_inner)), Some(AddChars(ref b_chars))) => {
                    t.regenerate(); // caret-35

                    t.chars_a(b_chars);

                    t.b_del.place(&DelSkip(b_chars.chars().count()));
                    t.b_add.place(&AddSkip(b_chars.chars().count()));

                    b.next();
                }
            }
        }
    }

    log_transform!("TRACK A DEL {:?}", t.a_del);
    log_transform!("TRACK A ADD {:?}", t.a_add);
    log_transform!("TRACK B DEL {:?}", t.b_del);
    log_transform!("TRACK B ADD {:?}", t.b_add);

    let (op_a, op_b) = t.result();
    log_transform!("RESULT A: {:?}", op_a.clone());
    log_transform!("RESULT B: {:?}", op_b.clone());
    (op_a, op_b)
}

fn undel(input_del: &DelSpan) -> DelSpan {
    let mut del: DelSpan = vec![];
    for elem in input_del {
        match elem {
            &DelChars(..) => {
                // skip
            }
            &DelSkip(value) => {
                del.place(&DelSkip(value));
            }
            &DelWithGroup(ref ins_span) => {
                del.place(&DelWithGroup(undel(ins_span)));
            }
            &DelGroup(ref del_span) => {
                del.place_all(&undel(&del_span));
            }
        }
    }
    del
}
            

pub fn transform_del_del_inner(
    a_del: &mut DelWriter,
    b_del: &mut DelWriter,
    a: &mut DelStepper,
    b: &mut DelStepper,
) {
    while !a.is_done() && !b.is_done() {
        log_transform!("{}", Green.bold().paint("transform_deletions:"));
        log_transform!("{}", BrightGreen.paint(format!(" @ a_del: {:?}", a_del)));
        log_transform!("{}", BrightGreen.paint(format!(" @ b_del: {:?}", b_del)));

        log_transform!(
            "{}",
            BrightYellow.paint(format!("Next step (del):\n A {:?}\n B {:?}", a.head.clone(), b.head.clone()))
        );

        match (a.head.clone(), b.head.clone()) {
            // Groups
            (Some(DelGroup(a_inner)), Some(DelGroup(b_inner))) => {
                let (a_del_inner, b_del_inner) = transform_deletions(&a_inner, &b_inner);

                a_del.place_all(&a_del_inner);
                b_del.place_all(&b_del_inner);

                a.next();
                b.next();
            }
            (Some(DelGroup(a_inner)), Some(DelWithGroup(b_inner))) => {
                let mut a_inner_del = DelWriter::new();
                let mut b_inner_del = DelWriter::new();

                let mut a_inner_step = DelStepper::new(&a_inner);
                let mut b_inner_step = DelStepper::new(&b_inner);

                transform_del_del_inner(
                    &mut a_inner_del,
                    &mut b_inner_del,
                    &mut a_inner_step,
                    &mut b_inner_step,
                );

                assert!(b_inner_step.is_done());

                // Del the del
                let mut del_span = vec![];
                while !a_inner_step.is_done() {
                    del_span.push(a_inner_step.head.clone().unwrap());
                    a_inner_step.next();
                }
                log_transform!("hello -----> {:?}", &del_span);
                a_inner_del.place_all(&undel(&del_span));
                b_inner_del.place_all(&del_span);


                a_del.place_all(&a_inner_del.result());
                b_del.place(&DelGroup(b_inner_del.result()));

                a.next();
                b.next();
            }
            (Some(DelSkip(a_count)), Some(DelGroup(b_inner))) => {
                a_del.place(&DelGroup(b_inner.clone()));
                if a_count > 1 {
                    a.head = Some(DelSkip(a_count - 1));
                } else {
                    a.next();
                }

                if b_inner.skip_post_len() > 0 {
                    b_del.place(&DelSkip(b_inner.skip_post_len()));
                }
                b.next();
            }
            (Some(DelGroup(a_inner)), Some(DelSkip(b_count))) => {
                if a_inner.skip_post_len() > 0 {
                    a_del.place(&DelSkip(a_inner.skip_post_len()));
                }
                b_del.place(&DelGroup(a_inner));

                a.next();
                if b_count > 1 {
                    b.head = Some(DelSkip(b_count - 1));
                } else {
                    b.next();
                }
            }

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

                a_del.place(&DelSkip(cmp::min(a_count, b_count)));
                b_del.place(&DelSkip(cmp::min(a_count, b_count)));
            }
            (Some(DelSkip(a_count)), Some(DelChars(b_chars))) => {
                if a_count > b_chars {
                    a.head = Some(DelSkip(a_count - b_chars));
                    b.next();
                    a_del.place(&DelChars(b_chars));
                } else if a_count < b_chars {
                    a.next();
                    b.head = Some(DelChars(b_chars - a_count));
                    a_del.place(&DelChars(a_count));
                } else {
                    a.next();
                    b.next();
                    a_del.place(&DelChars(b_chars));
                }
            }
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
            }
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
                b_del.place(&DelChars(cmp::min(a_chars, b_count)));
            }
            (Some(DelChars(a_chars)), _) => {
                a.next();
                b_del.place(&DelChars(a_chars));
            }

            // With Groups
            (Some(DelWithGroup(a_inner)), Some(DelWithGroup(b_inner))) => {
                let (a_del_inner, b_del_inner) = transform_deletions(&a_inner, &b_inner);

                a_del.place(&DelWithGroup(a_del_inner));
                b_del.place(&DelWithGroup(b_del_inner));

                a.next();
                b.next();
            }
            (Some(DelSkip(a_count)), Some(DelWithGroup(b_inner))) => {
                a_del.place(&DelWithGroup(b_inner));
                b_del.place(&DelSkip(1));

                if a_count > 1 {
                    a.head = Some(DelSkip(a_count - 1));
                } else {
                    a.next();
                }
                b.next();
            }
            (Some(DelWithGroup(a_inner)), Some(DelSkip(b_count))) => {
                a_del.place(&DelSkip(1));
                b_del.place(&DelWithGroup(a_inner));

                a.next();
                if b_count > 1 {
                    b.head = Some(DelSkip(b_count - 1));
                } else {
                    b.next();
                }
            }
            (Some(DelWithGroup(a_inner)), Some(DelGroup(b_inner))) => {
                let mut a_inner_del = DelWriter::new();
                let mut b_inner_del = DelWriter::new();

                let mut a_inner_step = DelStepper::new(&a_inner);
                let mut b_inner_step = DelStepper::new(&b_inner);

                transform_del_del_inner(
                    &mut a_inner_del,
                    &mut b_inner_del,
                    &mut a_inner_step,
                    &mut b_inner_step,
                );

                assert!(a_inner_step.is_done());
            
            
                // Del the del
                let mut del_span = vec![];
                while !b_inner_step.is_done() {
                    del_span.push(b_inner_step.head.clone().unwrap());
                    b_inner_step.next();
                }
                a_inner_del.place_all(&del_span);
                b_inner_del.place_all(&undel(&del_span));


                a_del.place(&DelGroup(a_inner_del.result()));
                b_del.place_all(&b_inner_del.result());

                a.next();
                b.next();
            }

            // TODO
            // (Some(DelGroupAll), Some(DelWithGroup(_))) => {
            //     b_del.group_all();

            //     a.next();
            //     b.next();
            // }
            // (Some(DelWithGroup(_)), Some(DelGroupAll)) => {
            //     a_del.group_all();

            //     a.next();
            //     b.next();
            // }
            // (Some(DelSkip(a_count)), Some(DelGroupAll)) => {
            //     a_del.group_all();
            //     if a_count > 1 {
            //         a.head = Some(DelSkip(a_count - 1));
            //     } else {
            //         a.next();
            //     }

            //     b.next();
            // }
            // (Some(DelGroupAll), Some(DelSkip(b_count))) => {
            //     b_del.group_all();

            //     a.next();
            //     if b_count > 1 {
            //         b.head = Some(DelSkip(b_count - 1));
            //     } else {
            //         b.next();
            //     }
            // }
            // (Some(DelGroupAll), Some(DelGroup(b_inner))) => {
            //     if b_inner.skip_post_len() > 0 {
            //         b_del.many(b_inner.skip_post_len());
            //     }

            //     a.next();
            //     b.next();
            // }
            // (Some(DelGroup(a_inner)), Some(DelGroupAll)) => {
            //     if a_inner.skip_post_len() > 0 {
            //         a_del.many(a_inner.skip_post_len());
            //     }

            //     a.next();
            //     b.next();
            // }
            // (Some(DelGroupAll), Some(DelGroupAll)) => {
            //     a.next();
            //     b.next();
            // }

            unimplemented => {
                log_transform!("Not reachable: {:?}", unimplemented);
                unreachable!();
            }
        }
    }

    log_transform!(
        "{}",
        BrightYellow.paint(format!("done")),
    );
}

pub fn transform_deletions(avec: &DelSpan, bvec: &DelSpan) -> (DelSpan, DelSpan) {
    let mut a_del = DelWriter::new();
    let mut b_del = DelWriter::new();

    let mut a = DelStepper::new(avec);
    let mut b = DelStepper::new(bvec);

    transform_del_del_inner(&mut a_del, &mut b_del, &mut a, &mut b);

    while !b.is_done() {
        log_transform!(
            "{}",
            BrightYellow.paint(format!("Finishing B: {:?}", b.head.clone()))
        );

        match b.head.clone() {
            Some(ref elem) => {
                a_del.place(elem);
                b.next();
            }
            None => {
                b.exit();
            }
        }
    }

    while !a.is_done() {
        log_transform!(
            "{}",
            BrightYellow.paint(format!("Finishing A (del): {:?}", a.head.clone()))
        );

        match a.head.clone() {
            Some(ref elem) => {
                a.next();
                b_del.place(elem);
            }
            None => {
                a.exit();
            }
        }
    }

    let a_res = a_del.result();
    let b_res = b_del.result();

    log_transform!("{}", BrightYellow.paint(format!("Result A: {:?}", a_res)));
    log_transform!("{}", BrightYellow.paint(format!("Result B: {:?}", b_res)));

    (a_res, b_res)
}

pub fn transform_add_del_inner(
    delres: &mut DelSpan,
    addres: &mut AddSpan,
    a: &mut AddStepper,
    b: &mut DelStepper,
) {
    while !b.is_done() && !a.is_done() {
        match b.get_head() {
            DelChars(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        addres.place(&AddChars(avalue.clone()));
                        delres.place(&DelSkip(avalue.len()));
                        a.next();
                    }
                    AddSkip(acount) => {
                        if bcount < acount {
                            a.head = Some(AddSkip(acount - bcount));
                            delres.place(&b.next().unwrap());
                        } else if bcount > acount {
                            a.next();
                            delres.place(&DelChars(acount));
                            b.head = Some(DelChars(bcount - acount));
                        } else {
                            a.next();
                            delres.place(&b.next().unwrap());
                        }
                    }
                    AddGroup(attrs, a_span) => {
                        let mut a_inner = AddStepper::new(&a_span);
                        let mut addres_inner: AddSpan = vec![];
                        let mut delres_inner: DelSpan = vec![];
                        transform_add_del_inner(
                            &mut delres_inner,
                            &mut addres_inner,
                            &mut a_inner,
                            b,
                        );
                        if !a_inner.is_done() {
                            addres_inner.place(&a_inner.head.unwrap());
                            addres_inner.place_all(&a_inner.rest);
                        }
                        addres.place(&AddGroup(attrs, addres_inner));
                        delres.place(&DelWithGroup(delres_inner));
                        a.next();
                    }
                    unknown => {
                        log_transform!("Compare: {:?} {:?}", DelChars(bcount), unknown);
                        panic!("Unimplemented or Unexpected");
                    }
                }
            }
            DelSkip(bcount) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        addres.place(&AddChars(avalue.clone()));
                        delres.place(&DelSkip(avalue.len()));
                        a.next();
                    }
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
                    }
                    AddWithGroup(..) => {
                        addres.place(&a.next().unwrap());
                        delres.place(&DelSkip(1));
                        if bcount == 1 {
                            b.next();
                        } else {
                            b.head = Some(DelSkip(bcount - 1));
                        }
                    }
                    AddGroup(attrs, a_span) => {
                        let mut a_inner = AddStepper::new(&a_span);
                        let mut addres_inner: AddSpan = vec![];
                        let mut delres_inner: DelSpan = vec![];
                        transform_add_del_inner(
                            &mut delres_inner,
                            &mut addres_inner,
                            &mut a_inner,
                            b,
                        );
                        if !a_inner.is_done() {
                            addres_inner.place(&a_inner.head.unwrap());
                            addres_inner.place_all(&a_inner.rest);
                        }
                        addres.place(&AddGroup(attrs, addres_inner));
                        delres.place(&DelWithGroup(delres_inner));
                        a.next();
                    }
                }
            }
            DelWithGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        delres.place(&DelSkip(avalue.chars().count()));
                        addres.place(&a.next().unwrap());
                    }
                    AddSkip(acount) => {
                        delres.place(&b.next().unwrap());
                        addres.place(&AddSkip(1));
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    }
                    AddWithGroup(insspan) => {
                        a.next();
                        b.next();

                        let (del, ins) = transform_add_del(&insspan, &span);
                        delres.place(&DelWithGroup(del));
                        addres.place(&AddWithGroup(ins));
                    }
                    AddGroup(attrs, a_span) => {
                        let mut a_inner = AddStepper::new(&a_span);
                        let mut addres_inner: AddSpan = vec![];
                        let mut delres_inner: DelSpan = vec![];
                        transform_add_del_inner(
                            &mut delres_inner,
                            &mut addres_inner,
                            &mut a_inner,
                            b,
                        );
                        if !a_inner.is_done() {
                            addres_inner.place(&a_inner.head.unwrap());
                            addres_inner.place_all(&a_inner.rest);
                        }
                        addres.place(&AddGroup(attrs, addres_inner));
                        delres.place(&DelWithGroup(delres_inner));
                        a.next();
                    }
                }
            }
            DelGroup(span) => {
                match a.get_head() {
                    AddChars(avalue) => {
                        delres.place(&DelSkip(avalue.chars().count()));
                        addres.place(&a.next().unwrap());
                    }
                    AddSkip(acount) => {
                        delres.place(&b.next().unwrap());
                        if span.skip_post_len() > 0 {
                            addres.place(&AddSkip(span.skip_post_len()));
                        }
                        if acount > 1 {
                            a.head = Some(AddSkip(acount - 1));
                        } else {
                            a.next();
                        }
                    }
                    AddWithGroup(ins_span) => {
                        if span.skip_post_len() == 0 {
                            fn unadd(add: &AddSpan) -> DelSpan {
                                let mut del: DelSpan = vec![];
                                for elem in add {
                                    match elem {
                                        &AddChars(ref value) => {
                                            del.place(&DelChars(value.chars().count()));
                                        }
                                        &AddSkip(value) => {
                                            del.place(&DelSkip(value));
                                        }
                                        &AddWithGroup(ref ins_span) => {
                                            del.place(&DelWithGroup(unadd(ins_span)));
                                        }
                                        &AddGroup(ref attrs, ref ins_span) => {
                                            del.place(&DelGroup(unadd(ins_span)));
                                        }
                                    }
                                }
                                del
                            }

                            // Undo any additions, then apply the complete deletion.
                            let del_span = compose::compose_del_del(&unadd(&ins_span), &span);
                            delres.place(&DelGroup(del_span));
                        } else {
                            let mut a_inner = AddStepper::new(&ins_span);
                            let mut b_inner = DelStepper::new(&span);
                            let mut delres_inner: DelSpan = vec![];
                            let mut addres_inner: AddSpan = vec![];
                            transform_add_del_inner(
                                &mut delres_inner,
                                &mut addres_inner,
                                &mut a_inner,
                                &mut b_inner,
                            );

                            // TODO should this be part of the top-level resolution for transform_add_del
                            // Finish consuming the Del or Add component
                            if !b_inner.is_done() {
                                if let &Some(ref head) = &b_inner.head {
                                    let len = (vec![head.clone()]).skip_post_len();
                                    if len > 0 {
                                        addres_inner.place(&AddSkip(len));
                                    }
                                }
                                let len = b_inner.rest.skip_post_len();
                                if len > 0 {
                                    addres_inner.place(&AddSkip(len));
                                }

                                delres_inner.place(&b_inner.head.unwrap());
                                delres_inner.place_all(&b_inner.rest);
                            } else if !a_inner.is_done() {

                                if let &Some(ref head) = &a_inner.head {
                                    let len = (vec![head.clone()]).skip_post_len();
                                    if len > 0 {
                                        delres_inner.place(&DelSkip(len));
                                    }
                                }
                                let len = a_inner.rest.skip_post_len();
                                if len > 0 {
                                    delres_inner.place(&DelSkip(len));
                                }

                                addres_inner.place(&a_inner.head.unwrap());
                                addres_inner.place_all(&a_inner.rest);
                            }

                            delres.place(&DelGroup(delres_inner));
                            addres.place_all(&addres_inner);
                        }

                        a.next();
                        b.next();
                    }

                    AddGroup(tags, ins_span) => {
                        let mut a_inner = AddStepper::new(&ins_span);
                        let mut delres_inner: DelSpan = vec![];
                        let mut addres_inner: AddSpan = vec![];
                        transform_add_del_inner(
                            &mut delres_inner,
                            &mut addres_inner,
                            &mut a_inner,
                            b,
                        );
                        if !a_inner.is_done() {
                            addres_inner.place(&a_inner.head.unwrap());
                            addres_inner.place_all(&a_inner.rest);
                        }
                        addres.place(&AddGroup(tags, addres_inner));
                        delres.place(&DelWithGroup(delres_inner));
                        a.next();
                    }
                }
            }
            // DelObject => {
            //     unimplemented!();
            // }
            // DelMany(bcount) => {
            //     match a.get_head() {
            //         AddChars(avalue) => {
            //             addres.place(&AddChars(avalue.clone()));
            //             delres.place(&DelSkip(avalue.len()));
            //             a.next();
            //         }
            //         AddSkip(acount) => {
            //             if bcount < acount {
            //                 a.head = Some(AddSkip(acount - bcount));
            //                 delres.place(&b.next().unwrap());
            //             } else if bcount > acount {
            //                 a.next();
            //                 delres.place(&DelMany(acount));
            //                 b.head = Some(DelMany(bcount - acount));
            //             } else {
            //                 a.next();
            //                 delres.place(&b.next().unwrap());
            //             }
            //         }
            //         AddGroup(attrs, a_span) => {
            //             let mut a_inner = AddStepper::new(&a_span);
            //             let mut addres_inner: AddSpan = vec![];
            //             let mut delres_inner: DelSpan = vec![];
            //             transform_add_del_inner(
            //                 &mut delres_inner,
            //                 &mut addres_inner,
            //                 &mut a_inner,
            //                 b,
            //             );
            //             if !a_inner.is_done() {
            //                 addres_inner.place(&a_inner.head.unwrap());
            //                 addres_inner.place_all(&a_inner.rest);
            //             }
            //             addres.place(&AddGroup(attrs, addres_inner));
            //             delres.place(&DelWithGroup(delres_inner));
            //             a.next();
            //         }
            //         AddWithGroup(ins_span) => {
            //             if bcount > 1 {
            //                 delres.place(&DelMany(1));
            //                 b.head = Some(DelMany(bcount - 1));
            //                 a.next();
            //             } else {
            //                 delres.place(&b.next().unwrap());
            //                 a.next();
            //             }
            //         }
            //     }
            // }
            // DelGroupAll => {
            //     match a.get_head() {
            //         AddChars(avalue) => {
            //             delres.place(&DelSkip(avalue.chars().count()));
            //             addres.place(&a.next().unwrap());
            //         }
            //         AddSkip(acount) => {
            //             delres.place(&b.next().unwrap());
            //             if acount > 1 {
            //                 a.head = Some(AddSkip(acount - 1));
            //             } else {
            //                 a.next();
            //             }
            //         }
            //         AddWithGroup(insspan) => {
            //             a.next();
            //             delres.place(&b.next().unwrap());
            //         }
            //         AddGroup(attrs, ins_span) => {
            //             let mut a_inner = AddStepper::new(&ins_span);
            //             let mut delres_inner: DelSpan = vec![];
            //             let mut addres_inner: AddSpan = vec![];
            //             transform_add_del_inner(
            //                 &mut delres_inner,
            //                 &mut addres_inner,
            //                 &mut a_inner,
            //                 b,
            //             );
            //             if !a_inner.is_done() {
            //                 addres_inner.place(&a_inner.head.unwrap());
            //                 addres_inner.place_all(&a_inner.rest);
            //             }
            //             addres.place(&AddGroup(attrs, addres_inner));
            //             delres.place(&DelWithGroup(delres_inner));

            //             log_transform!("NOW   ->{:?}\n   ->{:?}", addres, delres);
            //             a.next();
            //         }
            //     }
            // }
        }
    }
}

/// Transforms a insertion preceding a deletion into a deletion preceding an insertion.
/// After this, sequential deletions and insertions can be composed together in one operation.
pub fn transform_add_del(avec: &AddSpan, bvec: &DelSpan) -> Op {
    let mut delres: DelSpan = Vec::with_capacity(avec.len() + bvec.len());
    let mut addres: AddSpan = Vec::with_capacity(avec.len() + bvec.len());

    let mut a = AddStepper::new(avec);
    let mut b = DelStepper::new(bvec);

    transform_add_del_inner(&mut delres, &mut addres, &mut a, &mut b);

    if !b.is_done() {
        delres.place_all(&b.into_span());
    }

    if !a.is_done() {
        let rest = a.into_span();
        delres.place(&DelSkip(rest.skip_post_len()));
        addres.place_all(&rest);
    }

    (delres, addres)
}

/// Transform two operations according to a schema.
pub fn transform<S: Schema>(a: &Op, b: &Op) -> (Op, Op) {
    use super::schema::*;

    // Transform deletions A and B against each other to get delA` and delB`.
    log_transform!(" # transform[1] transform_deletions");
    log_transform!(" a_del   {:?}", a.0);
    log_transform!(" b_del   {:?}", b.0);
    log_transform!();

    let (a_del_0, b_del_0) = transform_deletions(&a.0, &b.0);
    log_transform!(" == a_del_0 {:?}", a_del_0);
    log_transform!(" == b_del_0 {:?}", b_del_0);
    log_transform!();

    // How do you apply del' if add has already been applied on the client?
    // The result will be applied after the client's insert operations had already been performed.
    // Reverse the impact of insA with delA` to not affect already newly added elements or text.
    log_transform!(" # transform[2] transform_add_del");
    log_transform!(" a_ins   {:?}", a.1);
    log_transform!(" a_del_0 {:?}", a_del_0);
    log_transform!(" ~ transform_add_del()");
    let (a_del_1, a_ins_1) = transform_add_del(&a.1, &a_del_0);
    log_transform!(" == a_del_1 {:?}", a_del_1);
    log_transform!(" == a_ins_1 {:?}", a_ins_1);
    log_transform!();

    log_transform!(" # transform[3] transform_add_del");
    log_transform!(" b_ins   {:?}", b.1);
    log_transform!(" b_del_0 {:?}", b_del_0);
    log_transform!(" ~ transform_add_del()");
    let (b_del_1, b_ins_1) = transform_add_del(&b.1, &b_del_0);
    log_transform!(" == b_del_1 {:?}", b_del_1);
    log_transform!(" == b_ins_1 {:?}", b_ins_1);
    log_transform!();

    // Insertions from both clients must be composed as though they happened against delA` and delB`
    // so that we don't have phantom elements.

    // Transform insert operations together.
    log_transform!(" # transform[4] transform_insertions");
    log_transform!(" a_ins_1 {:?}", a_ins_1);
    log_transform!(" b_ins_1 {:?}", b_ins_1);
    let ((a_del_2, a_ins_2), (b_del_2, b_ins_2)) = transform_insertions::<S>(&a_ins_1, &b_ins_1);
    log_transform!(" == a_del_2 {:?}", a_del_2);
    log_transform!(" == a_ins_2 {:?}", a_ins_2);
    log_transform!(" == b_del_2 {:?}", b_del_2);
    log_transform!(" == b_ins_2 {:?}", b_ins_2);
    log_transform!();

    // Our delete operations are now subsequent operations, and so can be composed.
    log_transform!(" # transform[5] compose_del_del");
    log_transform!(" a_del_1 {:?}", a_del_1);
    log_transform!(" a_del_2 {:?}", a_del_2);
    let a_del_3 = compose::compose_del_del(&a_del_1, &a_del_2);
    log_transform!(" == a_del_3 {:?}", a_del_3);
    log_transform!();
    log_transform!(" # transform[6] compose_del_del");
    log_transform!(" b_del_1 {:?}", b_del_1);
    log_transform!(" b_del_2 {:?}", b_del_2);
    let b_del_3 = compose::compose_del_del(&b_del_1, &b_del_2);
    log_transform!(" == b_del_3 {:?}", b_del_3);
    log_transform!();

    log_transform!(" # transform[result]");
    log_transform!(" a_del   {:?}", a.0);
    log_transform!(" a_ins   {:?}", a.1);
    log_transform!(" ~ transform()");
    log_transform!(" =a_del_3  {:?}", a_del_3);
    log_transform!(" =a_ins_2  {:?}", a_ins_2);
    log_transform!(" ---");
    log_transform!(" b_del   {:?}", b.0);
    log_transform!(" b_ins   {:?}", b.1);
    log_transform!(" ~ transform()");
    log_transform!(" =b_del_3  {:?}", b_del_3);
    log_transform!(" =b_ins_2  {:?}", b_ins_2);
    log_transform!();

    ((a_del_3, a_ins_2), (b_del_3, b_ins_2))
}
