(module
  (type (;0;) (func))
  (type (;1;) (func (param i32 i32 i32) (result i32)))
  (type (;2;) (func (param f32 f32) (result f32)))
  (type (;3;) (func (param f64 f64) (result f64)))
  (type (;4;) (func (param i32 i32) (result i32)))
  (type (;5;) (func (param i64 i64) (result i64)))
  (type (;6;) (func (param i64 i64 i32) (result i64)))
  (type (;7;) (func (param i32 i64 i64 i64 i64)))
  (type (;8;) (func (param i32 i64 i64 i64 i64 i32)))
  (type (;9;) (func (param f32 i32) (result f32)))
  (type (;10;) (func (param f64 i32) (result f64)))
  (type (;11;) (func (param i64 i32) (result i64)))
  (type (;12;) (func (param i32 i64 i64 i32)))
  (type (;13;) (func (param i32) (result f32)))
  (type (;14;) (func (param i32) (result f64)))
  (type (;15;) (func (param i64) (result f64)))
  (type (;16;) (func (param i64 i64) (result f32)))
  (type (;17;) (func (param i64 i64) (result f64)))
  (type (;18;) (func (param f32) (result i32)))
  (type (;19;) (func (param f32) (result i64)))
  (type (;20;) (func (param i32 f32)))
  (type (;21;) (func (param f64) (result i32)))
  (type (;22;) (func (param f64) (result i64)))
  (type (;23;) (func (param i32 f64)))
  (func $rust_eh_personality (type 0))
  (func $memcpy (type 1) (param i32 i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      get_local 2
      i32.eqz
      br_if 0 (;@1;)
      get_local 0
      set_local 3
      loop  ;; label = @2
        get_local 3
        get_local 1
        i32.load8_u
        i32.store8
        get_local 1
        i32.const 1
        i32.add
        set_local 1
        get_local 3
        i32.const 1
        i32.add
        set_local 3
        get_local 2
        i32.const -1
        i32.add
        tee_local 2
        br_if 0 (;@2;)
      end
    end
    get_local 0)
  (func $memmove (type 1) (param i32 i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      block  ;; label = @2
        get_local 1
        get_local 0
        i32.ge_u
        br_if 0 (;@2;)
        get_local 2
        i32.eqz
        br_if 1 (;@1;)
        loop  ;; label = @3
          get_local 0
          get_local 2
          i32.add
          i32.const -1
          i32.add
          get_local 1
          get_local 2
          i32.add
          i32.const -1
          i32.add
          i32.load8_u
          i32.store8
          get_local 2
          i32.const -1
          i32.add
          tee_local 2
          br_if 0 (;@3;)
          br 2 (;@1;)
        end
        unreachable
      end
      get_local 2
      i32.eqz
      br_if 0 (;@1;)
      get_local 0
      set_local 3
      loop  ;; label = @2
        get_local 3
        get_local 1
        i32.load8_u
        i32.store8
        get_local 1
        i32.const 1
        i32.add
        set_local 1
        get_local 3
        i32.const 1
        i32.add
        set_local 3
        get_local 2
        i32.const -1
        i32.add
        tee_local 2
        br_if 0 (;@2;)
      end
    end
    get_local 0)
  (func $memset (type 1) (param i32 i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      get_local 2
      i32.eqz
      br_if 0 (;@1;)
      get_local 0
      set_local 3
      loop  ;; label = @2
        get_local 3
        get_local 1
        i32.store8
        get_local 3
        i32.const 1
        i32.add
        set_local 3
        get_local 2
        i32.const -1
        i32.add
        tee_local 2
        br_if 0 (;@2;)
      end
    end
    get_local 0)
  (func $memcmp (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        get_local 2
        i32.eqz
        br_if 0 (;@2;)
        i32.const 0
        set_local 5
        loop  ;; label = @3
          get_local 0
          get_local 5
          i32.add
          i32.load8_u
          tee_local 3
          get_local 1
          get_local 5
          i32.add
          i32.load8_u
          tee_local 4
          i32.ne
          br_if 2 (;@1;)
          get_local 5
          i32.const 1
          i32.add
          tee_local 5
          get_local 2
          i32.lt_u
          br_if 0 (;@3;)
        end
        i32.const 0
        return
      end
      i32.const 0
      return
    end
    get_local 3
    get_local 4
    i32.sub)
  (func $__subsf3 (type 2) (param f32 f32) (result f32)
    get_local 1
    i32.reinterpret/f32
    i32.const -2147483648
    i32.xor
    f32.reinterpret/i32
    get_local 0
    f32.add)
  (func $__subdf3 (type 3) (param f64 f64) (result f64)
    get_local 1
    i64.reinterpret/f64
    i64.const -9223372036854775808
    i64.xor
    f64.reinterpret/i64
    get_local 0
    f64.add)
  (func $__udivsi3 (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32)
    block  ;; label = @1
      get_local 1
      i32.eqz
      br_if 0 (;@1;)
      i32.const 0
      set_local 5
      block  ;; label = @2
        get_local 0
        i32.eqz
        br_if 0 (;@2;)
        get_local 1
        i32.clz
        get_local 0
        i32.clz
        i32.sub
        tee_local 4
        i32.const 32
        i32.ge_u
        br_if 0 (;@2;)
        block  ;; label = @3
          get_local 4
          i32.const 31
          i32.ne
          br_if 0 (;@3;)
          get_local 0
          return
        end
        get_local 0
        i32.const 31
        get_local 4
        i32.sub
        i32.const 31
        i32.and
        i32.shl
        set_local 5
        block  ;; label = @3
          block  ;; label = @4
            get_local 4
            i32.const 1
            i32.add
            tee_local 4
            i32.eqz
            br_if 0 (;@4;)
            get_local 1
            i32.const -1
            i32.add
            set_local 2
            get_local 0
            get_local 4
            i32.const 31
            i32.and
            i32.shr_u
            set_local 0
            i32.const 0
            set_local 6
            loop  ;; label = @5
              get_local 5
              i32.const 31
              i32.shr_u
              get_local 0
              i32.const 1
              i32.shl
              i32.or
              tee_local 0
              get_local 2
              get_local 0
              i32.sub
              i32.const 31
              i32.shr_s
              tee_local 3
              get_local 1
              i32.and
              i32.sub
              set_local 0
              get_local 5
              i32.const 1
              i32.shl
              get_local 6
              i32.or
              set_local 5
              get_local 3
              i32.const 1
              i32.and
              tee_local 3
              set_local 6
              get_local 4
              i32.const -1
              i32.add
              tee_local 4
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
            unreachable
          end
          i32.const 0
          set_local 3
        end
        get_local 5
        i32.const 1
        i32.shl
        get_local 3
        i32.or
        set_local 5
      end
      get_local 5
      return
    end
    unreachable
    unreachable)
  (func $__umodsi3 (type 4) (param i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      get_local 1
      i32.eqz
      br_if 0 (;@1;)
      i32.const 0
      set_local 5
      block  ;; label = @2
        get_local 0
        i32.eqz
        br_if 0 (;@2;)
        get_local 1
        i32.clz
        get_local 0
        i32.clz
        i32.sub
        tee_local 4
        i32.const 31
        i32.gt_u
        br_if 0 (;@2;)
        get_local 0
        set_local 5
        get_local 4
        i32.const 31
        i32.eq
        br_if 0 (;@2;)
        get_local 0
        i32.const 31
        get_local 4
        i32.sub
        i32.const 31
        i32.and
        i32.shl
        set_local 5
        block  ;; label = @3
          block  ;; label = @4
            get_local 4
            i32.const 1
            i32.add
            tee_local 4
            i32.eqz
            br_if 0 (;@4;)
            get_local 1
            i32.const -1
            i32.add
            set_local 2
            get_local 0
            get_local 4
            i32.const 31
            i32.and
            i32.shr_u
            set_local 6
            i32.const 0
            set_local 7
            loop  ;; label = @5
              get_local 6
              i32.const 1
              i32.shl
              get_local 5
              i32.const 31
              i32.shr_u
              i32.or
              tee_local 6
              get_local 2
              get_local 6
              i32.sub
              i32.const 31
              i32.shr_s
              tee_local 3
              get_local 1
              i32.and
              i32.sub
              set_local 6
              get_local 7
              get_local 5
              i32.const 1
              i32.shl
              i32.or
              set_local 5
              get_local 3
              i32.const 1
              i32.and
              tee_local 3
              set_local 7
              get_local 4
              i32.const -1
              i32.add
              tee_local 4
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
            unreachable
          end
          i32.const 0
          set_local 3
        end
        get_local 5
        i32.const 1
        i32.shl
        get_local 3
        i32.or
        set_local 5
      end
      get_local 0
      get_local 5
      get_local 1
      i32.mul
      i32.sub
      return
    end
    unreachable
    unreachable)
  (func $__udivmodsi4 (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      get_local 1
      i32.eqz
      br_if 0 (;@1;)
      i32.const 0
      set_local 6
      block  ;; label = @2
        get_local 0
        i32.eqz
        br_if 0 (;@2;)
        get_local 1
        i32.clz
        get_local 0
        i32.clz
        i32.sub
        tee_local 5
        i32.const 31
        i32.gt_u
        br_if 0 (;@2;)
        get_local 0
        set_local 6
        get_local 5
        i32.const 31
        i32.eq
        br_if 0 (;@2;)
        get_local 0
        i32.const 31
        get_local 5
        i32.sub
        i32.const 31
        i32.and
        i32.shl
        set_local 6
        block  ;; label = @3
          block  ;; label = @4
            get_local 5
            i32.const 1
            i32.add
            tee_local 5
            i32.eqz
            br_if 0 (;@4;)
            get_local 1
            i32.const -1
            i32.add
            set_local 3
            get_local 0
            get_local 5
            i32.const 31
            i32.and
            i32.shr_u
            set_local 7
            i32.const 0
            set_local 8
            loop  ;; label = @5
              get_local 7
              i32.const 1
              i32.shl
              get_local 6
              i32.const 31
              i32.shr_u
              i32.or
              tee_local 7
              get_local 3
              get_local 7
              i32.sub
              i32.const 31
              i32.shr_s
              tee_local 4
              get_local 1
              i32.and
              i32.sub
              set_local 7
              get_local 8
              get_local 6
              i32.const 1
              i32.shl
              i32.or
              set_local 6
              get_local 4
              i32.const 1
              i32.and
              tee_local 4
              set_local 8
              get_local 5
              i32.const -1
              i32.add
              tee_local 5
              br_if 0 (;@5;)
              br 2 (;@3;)
            end
            unreachable
          end
          i32.const 0
          set_local 4
        end
        get_local 6
        i32.const 1
        i32.shl
        get_local 4
        i32.or
        set_local 6
      end
      block  ;; label = @2
        get_local 2
        i32.eqz
        br_if 0 (;@2;)
        get_local 2
        get_local 0
        get_local 6
        get_local 1
        i32.mul
        i32.sub
        i32.store
      end
      get_local 6
      return
    end
    unreachable
    unreachable)
  (func $__udivdi3 (type 5) (param i64 i64) (result i64)
    get_local 0
    get_local 1
    i32.const 0
    call $__udivmoddi4)
  (func $__udivmoddi4 (type 6) (param i64 i64 i32) (result i64)
    (local i32 i32 i64 i64 i64 i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  block  ;; label = @16
                                    get_local 0
                                    i64.const 4294967295
                                    i64.gt_u
                                    br_if 0 (;@16;)
                                    get_local 1
                                    i64.const 4294967296
                                    i64.ge_u
                                    br_if 1 (;@15;)
                                    get_local 1
                                    i32.wrap/i64
                                    set_local 3
                                    get_local 2
                                    i32.eqz
                                    br_if 4 (;@12;)
                                    get_local 3
                                    i32.eqz
                                    br_if 13 (;@3;)
                                    get_local 2
                                    get_local 0
                                    i32.wrap/i64
                                    get_local 3
                                    i32.rem_u
                                    i64.extend_u/i32
                                    i64.store
                                    br 5 (;@11;)
                                  end
                                  get_local 1
                                  i32.wrap/i64
                                  tee_local 3
                                  i32.eqz
                                  br_if 1 (;@14;)
                                  get_local 1
                                  i64.const 4294967296
                                  i64.ge_u
                                  br_if 2 (;@13;)
                                  get_local 3
                                  i32.const -1
                                  i32.add
                                  get_local 3
                                  i32.and
                                  i32.eqz
                                  br_if 7 (;@8;)
                                  i32.const 0
                                  get_local 3
                                  i32.clz
                                  i32.const 33
                                  i32.add
                                  get_local 0
                                  i64.const 32
                                  i64.shr_u
                                  i32.wrap/i64
                                  i32.clz
                                  i32.sub
                                  tee_local 3
                                  i32.sub
                                  set_local 4
                                  br 11 (;@4;)
                                end
                                i64.const 0
                                set_local 8
                                get_local 2
                                i32.eqz
                                br_if 13 (;@1;)
                                get_local 2
                                get_local 0
                                i64.store
                                i64.const 0
                                return
                              end
                              get_local 1
                              i64.const 4294967296
                              i64.lt_u
                              br_if 10 (;@3;)
                              get_local 0
                              i32.wrap/i64
                              i32.eqz
                              br_if 4 (;@9;)
                              get_local 1
                              i64.const 32
                              i64.shr_u
                              i32.wrap/i64
                              tee_local 3
                              i32.eqz
                              br_if 6 (;@7;)
                              get_local 3
                              i32.const -1
                              i32.add
                              get_local 3
                              i32.and
                              br_if 6 (;@7;)
                              block  ;; label = @14
                                get_local 2
                                i32.eqz
                                br_if 0 (;@14;)
                                get_local 2
                                get_local 1
                                i64.const -4294967296
                                i64.add
                                i64.const 4294967295
                                i64.or
                                get_local 0
                                i64.and
                                i64.store
                              end
                              get_local 0
                              i64.const 32
                              i64.shr_u
                              i32.wrap/i64
                              get_local 3
                              i32.ctz
                              i32.const 31
                              i32.and
                              i32.shr_u
                              i64.extend_u/i32
                              return
                            end
                            get_local 1
                            i64.const 32
                            i64.shr_u
                            i32.wrap/i64
                            i32.clz
                            get_local 0
                            i64.const 32
                            i64.shr_u
                            i32.wrap/i64
                            i32.clz
                            i32.sub
                            tee_local 3
                            i32.const 31
                            i32.le_u
                            br_if 2 (;@10;)
                            i64.const 0
                            set_local 8
                            get_local 2
                            i32.eqz
                            br_if 11 (;@1;)
                            get_local 2
                            get_local 0
                            i64.store
                            i64.const 0
                            return
                          end
                          get_local 3
                          i32.eqz
                          br_if 9 (;@2;)
                        end
                        get_local 0
                        i32.wrap/i64
                        get_local 3
                        i32.div_u
                        i64.extend_u/i32
                        set_local 8
                        br 9 (;@1;)
                      end
                      i32.const 63
                      get_local 3
                      i32.sub
                      set_local 4
                      get_local 3
                      i32.const 1
                      i32.add
                      set_local 3
                      br 5 (;@4;)
                    end
                    get_local 1
                    i64.const 32
                    i64.shr_u
                    i32.wrap/i64
                    set_local 3
                    block  ;; label = @9
                      get_local 2
                      i32.eqz
                      br_if 0 (;@9;)
                      get_local 2
                      get_local 0
                      i64.const 32
                      i64.shr_u
                      i32.wrap/i64
                      get_local 3
                      i32.rem_u
                      i64.extend_u/i32
                      i64.const 32
                      i64.shl
                      i64.store
                    end
                    get_local 0
                    i64.const 32
                    i64.shr_u
                    i32.wrap/i64
                    get_local 3
                    i32.div_u
                    i64.extend_u/i32
                    return
                  end
                  block  ;; label = @8
                    get_local 2
                    i32.eqz
                    br_if 0 (;@8;)
                    get_local 2
                    get_local 1
                    i64.const 4294967295
                    i64.add
                    get_local 0
                    i64.and
                    i64.const 4294967295
                    i64.and
                    i64.store
                  end
                  get_local 3
                  i32.const 1
                  i32.ne
                  br_if 1 (;@6;)
                  get_local 0
                  return
                end
                get_local 3
                i32.clz
                get_local 0
                i64.const 32
                i64.shr_u
                i32.wrap/i64
                i32.clz
                i32.sub
                tee_local 3
                i32.const 30
                i32.le_u
                br_if 1 (;@5;)
                i64.const 0
                set_local 8
                get_local 2
                i32.eqz
                br_if 5 (;@1;)
                get_local 2
                get_local 0
                i64.store
                i64.const 0
                return
              end
              get_local 0
              get_local 3
              i32.ctz
              i64.extend_u/i32
              i64.shr_u
              return
            end
            i32.const 63
            get_local 3
            i32.sub
            set_local 4
            get_local 3
            i32.const 1
            i32.add
            set_local 3
          end
          get_local 0
          get_local 4
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shl
          set_local 8
          get_local 0
          get_local 3
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shr_u
          set_local 0
          block  ;; label = @4
            block  ;; label = @5
              get_local 3
              i32.eqz
              br_if 0 (;@5;)
              get_local 1
              i64.const -1
              i64.add
              set_local 5
              i64.const 0
              set_local 7
              loop  ;; label = @6
                get_local 0
                i64.const 1
                i64.shl
                get_local 8
                i64.const 63
                i64.shr_u
                i64.or
                tee_local 0
                get_local 5
                get_local 0
                i64.sub
                i64.const 63
                i64.shr_s
                tee_local 6
                get_local 1
                i64.and
                i64.sub
                set_local 0
                get_local 7
                get_local 8
                i64.const 1
                i64.shl
                i64.or
                set_local 8
                get_local 6
                i64.const 1
                i64.and
                tee_local 6
                set_local 7
                get_local 3
                i32.const -1
                i32.add
                tee_local 3
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
              unreachable
            end
            i64.const 0
            set_local 6
          end
          block  ;; label = @4
            get_local 2
            i32.eqz
            br_if 0 (;@4;)
            get_local 2
            get_local 0
            i64.store
          end
          get_local 6
          get_local 8
          i64.const 1
          i64.shl
          i64.or
          return
        end
        unreachable
        unreachable
      end
      unreachable
      unreachable
    end
    get_local 8)
  (func $__umoddi3 (type 5) (param i64 i64) (result i64)
    (local i32)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 2
    i32.store offset=4
    get_local 2
    i64.const 0
    i64.store offset=8
    get_local 0
    get_local 1
    get_local 2
    i32.const 8
    i32.add
    call $__udivmoddi4
    drop
    get_local 2
    i64.load offset=8
    set_local 0
    i32.const 0
    get_local 2
    i32.const 16
    i32.add
    i32.store offset=4
    get_local 0)
  (func $__udivti3 (type 7) (param i32 i64 i64 i64 i64)
    (local i32)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 5
    i32.store offset=4
    get_local 5
    get_local 1
    get_local 2
    get_local 3
    get_local 4
    i32.const 0
    call $__udivmodti4
    get_local 5
    i64.load
    set_local 1
    get_local 0
    i32.const 8
    i32.add
    get_local 5
    i32.const 8
    i32.add
    i64.load
    i64.store
    get_local 0
    get_local 1
    i64.store
    i32.const 0
    get_local 5
    i32.const 16
    i32.add
    i32.store offset=4)
  (func $__udivmodti4 (type 8) (param i32 i64 i64 i64 i64 i32)
    (local i32 i32 i32 i64 i64 i64 i64 i64 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 8
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            block  ;; label = @13
                              block  ;; label = @14
                                block  ;; label = @15
                                  block  ;; label = @16
                                    block  ;; label = @17
                                      get_local 2
                                      i64.const 0
                                      i64.ne
                                      br_if 0 (;@17;)
                                      get_local 4
                                      i64.eqz
                                      i32.eqz
                                      br_if 1 (;@16;)
                                      get_local 5
                                      i32.eqz
                                      br_if 4 (;@13;)
                                      get_local 3
                                      i64.const 0
                                      i64.eq
                                      br_if 15 (;@2;)
                                      get_local 5
                                      i32.const 8
                                      i32.add
                                      i64.const 0
                                      i64.store
                                      get_local 5
                                      get_local 1
                                      get_local 3
                                      i64.rem_u
                                      i64.store
                                      br 5 (;@12;)
                                    end
                                    get_local 3
                                    i64.eqz
                                    br_if 1 (;@15;)
                                    get_local 4
                                    i64.eqz
                                    i32.eqz
                                    br_if 2 (;@14;)
                                    get_local 3
                                    i64.const -1
                                    i64.add
                                    tee_local 13
                                    get_local 3
                                    i64.and
                                    i64.eqz
                                    br_if 7 (;@9;)
                                    i32.const 0
                                    get_local 3
                                    i64.clz
                                    i32.wrap/i64
                                    i32.const 65
                                    i32.add
                                    get_local 2
                                    i64.clz
                                    i32.wrap/i64
                                    i32.sub
                                    tee_local 6
                                    i32.sub
                                    set_local 7
                                    br 12 (;@4;)
                                  end
                                  get_local 5
                                  i32.eqz
                                  br_if 8 (;@7;)
                                  get_local 5
                                  get_local 1
                                  i64.store
                                  get_local 5
                                  i32.const 8
                                  i32.add
                                  get_local 2
                                  i64.store
                                  br 8 (;@7;)
                                end
                                get_local 4
                                i64.eqz
                                tee_local 6
                                br_if 12 (;@2;)
                                get_local 1
                                i64.const 0
                                i64.eq
                                br_if 4 (;@10;)
                                get_local 6
                                br_if 6 (;@8;)
                                get_local 4
                                i64.const -1
                                i64.add
                                tee_local 13
                                get_local 4
                                i64.and
                                i64.eqz
                                i32.eqz
                                br_if 6 (;@8;)
                                block  ;; label = @15
                                  get_local 5
                                  i32.eqz
                                  br_if 0 (;@15;)
                                  get_local 5
                                  get_local 1
                                  i64.store
                                  get_local 5
                                  i32.const 8
                                  i32.add
                                  get_local 13
                                  get_local 2
                                  i64.and
                                  i64.store
                                end
                                get_local 2
                                get_local 4
                                i64.ctz
                                i64.const 63
                                i64.and
                                i64.shr_u
                                set_local 1
                                br 8 (;@6;)
                              end
                              get_local 4
                              i64.clz
                              i32.wrap/i64
                              get_local 2
                              i64.clz
                              i32.wrap/i64
                              i32.sub
                              tee_local 6
                              i32.const 63
                              i32.le_u
                              br_if 2 (;@11;)
                              get_local 5
                              i32.eqz
                              br_if 6 (;@7;)
                              get_local 5
                              get_local 1
                              i64.store
                              get_local 5
                              i32.const 8
                              i32.add
                              get_local 2
                              i64.store
                              br 6 (;@7;)
                            end
                            get_local 3
                            i64.const 0
                            i64.eq
                            br_if 11 (;@1;)
                          end
                          get_local 1
                          get_local 3
                          i64.div_u
                          set_local 1
                          br 5 (;@6;)
                        end
                        i32.const 127
                        get_local 6
                        i32.sub
                        set_local 7
                        get_local 6
                        i32.const 1
                        i32.add
                        set_local 6
                        br 6 (;@4;)
                      end
                      block  ;; label = @10
                        get_local 5
                        i32.eqz
                        br_if 0 (;@10;)
                        get_local 5
                        i64.const 0
                        i64.store
                        get_local 5
                        i32.const 8
                        i32.add
                        get_local 2
                        get_local 4
                        i64.rem_u
                        i64.store
                      end
                      get_local 2
                      get_local 4
                      i64.div_u
                      set_local 1
                      br 3 (;@6;)
                    end
                    block  ;; label = @9
                      get_local 5
                      i32.eqz
                      br_if 0 (;@9;)
                      get_local 5
                      i32.const 8
                      i32.add
                      i64.const 0
                      i64.store
                      get_local 5
                      get_local 13
                      get_local 1
                      i64.and
                      i64.store
                    end
                    get_local 3
                    i64.const 1
                    i64.eq
                    br_if 5 (;@3;)
                    get_local 8
                    i32.const 32
                    i32.add
                    get_local 1
                    get_local 2
                    get_local 3
                    i64.ctz
                    i32.wrap/i64
                    call $__lshrti3
                    get_local 8
                    i32.const 40
                    i32.add
                    i64.load
                    set_local 2
                    get_local 8
                    i64.load offset=32
                    set_local 1
                    br 5 (;@3;)
                  end
                  get_local 4
                  i64.clz
                  i32.wrap/i64
                  get_local 2
                  i64.clz
                  i32.wrap/i64
                  i32.sub
                  tee_local 6
                  i32.const 62
                  i32.le_u
                  br_if 2 (;@5;)
                  get_local 5
                  i32.eqz
                  br_if 0 (;@7;)
                  get_local 5
                  get_local 1
                  i64.store
                  get_local 5
                  i32.const 8
                  i32.add
                  get_local 2
                  i64.store
                end
                i64.const 0
                set_local 1
              end
              i64.const 0
              set_local 2
              br 2 (;@3;)
            end
            i32.const 127
            get_local 6
            i32.sub
            set_local 7
            get_local 6
            i32.const 1
            i32.add
            set_local 6
          end
          get_local 8
          get_local 1
          get_local 2
          get_local 7
          i32.const 127
          i32.and
          call $__ashlti3
          get_local 8
          i32.const 16
          i32.add
          get_local 1
          get_local 2
          get_local 6
          i32.const 127
          i32.and
          call $__lshrti3
          get_local 8
          i32.const 8
          i32.add
          i64.load
          set_local 2
          get_local 8
          i32.const 16
          i32.add
          i32.const 8
          i32.add
          i64.load
          set_local 14
          get_local 8
          i64.load
          set_local 1
          get_local 8
          i64.load offset=16
          set_local 13
          block  ;; label = @4
            block  ;; label = @5
              get_local 6
              i32.eqz
              br_if 0 (;@5;)
              get_local 4
              i64.const 1
              get_local 3
              i64.const -1
              i64.add
              tee_local 9
              get_local 3
              i64.lt_u
              i64.extend_u/i32
              get_local 9
              i64.const -1
              i64.ne
              select
              i64.add
              i64.const -1
              i64.add
              set_local 10
              i64.const 0
              set_local 15
              i64.const 0
              set_local 16
              loop  ;; label = @6
                get_local 14
                i64.const 1
                i64.shl
                get_local 13
                i64.const 63
                i64.shr_u
                i64.or
                tee_local 11
                get_local 10
                get_local 11
                i64.sub
                get_local 9
                get_local 13
                i64.const 1
                i64.shl
                get_local 2
                i64.const 63
                i64.shr_u
                i64.or
                tee_local 13
                i64.lt_u
                i64.extend_u/i32
                i64.sub
                i64.const 63
                i64.shr_s
                tee_local 11
                get_local 4
                i64.and
                i64.sub
                get_local 13
                get_local 11
                get_local 3
                i64.and
                tee_local 12
                i64.lt_u
                i64.extend_u/i32
                i64.sub
                set_local 14
                get_local 13
                get_local 12
                i64.sub
                set_local 13
                i64.const 0
                get_local 2
                i64.const 1
                i64.shl
                get_local 1
                i64.const 63
                i64.shr_u
                i64.or
                i64.or
                set_local 2
                get_local 16
                get_local 1
                i64.const 1
                i64.shl
                i64.or
                set_local 1
                get_local 11
                i64.const 1
                i64.and
                tee_local 11
                set_local 16
                get_local 6
                i32.const -1
                i32.add
                tee_local 6
                br_if 0 (;@6;)
                br 2 (;@4;)
              end
              unreachable
            end
            i64.const 0
            set_local 11
            i64.const 0
            set_local 15
          end
          block  ;; label = @4
            get_local 5
            i32.eqz
            br_if 0 (;@4;)
            get_local 5
            get_local 13
            i64.store
            get_local 5
            i32.const 8
            i32.add
            get_local 14
            i64.store
          end
          get_local 15
          get_local 2
          i64.const 1
          i64.shl
          get_local 1
          i64.const 63
          i64.shr_u
          i64.or
          i64.or
          set_local 2
          get_local 11
          get_local 1
          i64.const 1
          i64.shl
          i64.or
          set_local 1
        end
        get_local 0
        get_local 1
        i64.store
        get_local 0
        i32.const 8
        i32.add
        get_local 2
        i64.store
        i32.const 0
        get_local 8
        i32.const 48
        i32.add
        i32.store offset=4
        return
      end
      unreachable
      unreachable
    end
    unreachable
    unreachable)
  (func $__umodti3 (type 7) (param i32 i64 i64 i64 i64)
    (local i32 i32)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 32
    i32.sub
    tee_local 6
    i32.store offset=4
    get_local 6
    i32.const 16
    i32.add
    i32.const 8
    i32.add
    tee_local 5
    i64.const 0
    i64.store
    get_local 6
    i64.const 0
    i64.store offset=16
    get_local 6
    get_local 1
    get_local 2
    get_local 3
    get_local 4
    get_local 6
    i32.const 16
    i32.add
    call $__udivmodti4
    get_local 6
    i64.load offset=16
    set_local 1
    get_local 0
    i32.const 8
    i32.add
    get_local 5
    i64.load
    i64.store
    get_local 0
    get_local 1
    i64.store
    i32.const 0
    get_local 6
    i32.const 32
    i32.add
    i32.store offset=4)
  (func $__addsf3 (type 2) (param f32 f32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32)
    get_local 1
    i32.reinterpret/f32
    tee_local 3
    i32.const 2147483647
    i32.and
    set_local 5
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i32.reinterpret/f32
              tee_local 2
              i32.const 2147483647
              i32.and
              tee_local 4
              i32.const -1
              i32.add
              i32.const 2139095038
              i32.le_u
              br_if 0 (;@5;)
              get_local 4
              i32.const 2139095041
              i32.lt_u
              br_if 1 (;@4;)
              get_local 4
              i32.const 4194304
              i32.or
              f32.reinterpret/i32
              return
            end
            get_local 5
            i32.const -1
            i32.add
            i32.const 2139095038
            i32.le_u
            br_if 1 (;@3;)
          end
          block  ;; label = @4
            get_local 5
            i32.const 2139095041
            i32.lt_u
            br_if 0 (;@4;)
            get_local 5
            i32.const 4194304
            i32.or
            f32.reinterpret/i32
            return
          end
          block  ;; label = @4
            get_local 4
            i32.const 2139095040
            i32.ne
            br_if 0 (;@4;)
            f32.const nan (;=nan;)
            get_local 0
            get_local 3
            get_local 2
            i32.xor
            i32.const -2147483648
            i32.eq
            select
            return
          end
          get_local 5
          i32.const 2139095040
          i32.eq
          br_if 2 (;@1;)
          get_local 4
          i32.eqz
          br_if 1 (;@2;)
          get_local 0
          set_local 1
          get_local 5
          i32.eqz
          br_if 2 (;@1;)
        end
        get_local 3
        get_local 2
        get_local 5
        get_local 4
        i32.gt_u
        tee_local 5
        select
        tee_local 4
        i32.const 8388607
        i32.and
        set_local 8
        get_local 2
        get_local 3
        get_local 5
        select
        tee_local 6
        i32.const 23
        i32.shr_u
        i32.const 255
        i32.and
        set_local 3
        block  ;; label = @3
          get_local 4
          i32.const 23
          i32.shr_u
          i32.const 255
          i32.and
          tee_local 5
          br_if 0 (;@3;)
          i32.const 9
          get_local 8
          i32.clz
          tee_local 2
          i32.sub
          set_local 5
          get_local 8
          get_local 2
          i32.const 24
          i32.add
          i32.const 31
          i32.and
          i32.shl
          set_local 8
        end
        get_local 6
        i32.const 8388607
        i32.and
        set_local 2
        block  ;; label = @3
          get_local 3
          br_if 0 (;@3;)
          i32.const 9
          get_local 2
          i32.clz
          tee_local 7
          i32.sub
          set_local 3
          get_local 2
          get_local 7
          i32.const 24
          i32.add
          i32.const 31
          i32.and
          i32.shl
          set_local 2
        end
        get_local 6
        get_local 4
        i32.xor
        set_local 6
        get_local 2
        i32.const 3
        i32.shl
        i32.const 67108864
        i32.or
        set_local 7
        get_local 8
        i32.const 3
        i32.shl
        set_local 8
        block  ;; label = @3
          block  ;; label = @4
            get_local 5
            get_local 3
            i32.sub
            tee_local 3
            i32.eqz
            br_if 0 (;@4;)
            i32.const 1
            set_local 2
            get_local 3
            i32.const 31
            i32.gt_u
            br_if 1 (;@3;)
            get_local 7
            get_local 3
            i32.const 31
            i32.and
            i32.shr_u
            get_local 7
            i32.const 0
            get_local 3
            i32.sub
            i32.const 31
            i32.and
            i32.shl
            i32.const 0
            i32.ne
            i32.or
            set_local 2
            br 1 (;@3;)
          end
          get_local 7
          set_local 2
        end
        get_local 8
        i32.const 67108864
        i32.or
        set_local 3
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 6
              i32.const -1
              i32.le_s
              br_if 0 (;@5;)
              get_local 2
              get_local 3
              i32.add
              tee_local 3
              i32.const 134217728
              i32.and
              i32.eqz
              br_if 1 (;@4;)
              get_local 2
              get_local 8
              i32.add
              i32.const 1
              i32.and
              get_local 3
              i32.const 1
              i32.shr_u
              i32.or
              set_local 3
              get_local 5
              i32.const 1
              i32.add
              set_local 5
              br 1 (;@4;)
            end
            get_local 3
            get_local 2
            i32.sub
            tee_local 3
            i32.eqz
            br_if 1 (;@3;)
            get_local 3
            i32.const 67108863
            i32.gt_u
            br_if 0 (;@4;)
            get_local 5
            get_local 3
            i32.clz
            i32.const -5
            i32.add
            tee_local 2
            i32.sub
            set_local 5
            get_local 3
            get_local 2
            i32.const 31
            i32.and
            i32.shl
            set_local 3
          end
          get_local 4
          i32.const -2147483648
          i32.and
          set_local 4
          block  ;; label = @4
            get_local 5
            i32.const 255
            i32.lt_s
            br_if 0 (;@4;)
            get_local 4
            i32.const 2139095040
            i32.or
            f32.reinterpret/i32
            return
          end
          i32.const 0
          set_local 2
          block  ;; label = @4
            block  ;; label = @5
              get_local 5
              i32.const 0
              i32.le_s
              br_if 0 (;@5;)
              get_local 5
              set_local 2
              br 1 (;@4;)
            end
            get_local 3
            i32.const 1
            get_local 5
            i32.sub
            tee_local 5
            i32.const 31
            i32.and
            i32.shr_u
            get_local 3
            i32.const 0
            get_local 5
            i32.sub
            i32.const 31
            i32.and
            i32.shl
            i32.const 0
            i32.ne
            i32.or
            set_local 3
          end
          get_local 3
          i32.const 3
          i32.shr_u
          tee_local 8
          i32.const 8388607
          i32.and
          get_local 4
          i32.or
          get_local 2
          i32.const 23
          i32.shl
          i32.or
          set_local 5
          block  ;; label = @4
            block  ;; label = @5
              get_local 3
              i32.const 7
              i32.and
              tee_local 4
              i32.const 5
              i32.lt_u
              br_if 0 (;@5;)
              get_local 5
              i32.const 1
              i32.add
              set_local 5
              br 1 (;@4;)
            end
            get_local 4
            i32.const 4
            i32.ne
            br_if 0 (;@4;)
            get_local 5
            get_local 8
            i32.const 1
            i32.and
            i32.add
            set_local 5
          end
          get_local 5
          f32.reinterpret/i32
          set_local 1
          br 2 (;@1;)
        end
        f32.const 0x0p+0 (;=0;)
        return
      end
      get_local 5
      br_if 0 (;@1;)
      get_local 3
      get_local 2
      i32.and
      f32.reinterpret/i32
      return
    end
    get_local 1)
  (func $__adddf3 (type 3) (param f64 f64) (result f64)
    (local i32 i32 i64 i64 i64 i64 i64 i64)
    get_local 1
    i64.reinterpret/f64
    tee_local 5
    i64.const 9223372036854775807
    i64.and
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i64.reinterpret/f64
              tee_local 4
              i64.const 9223372036854775807
              i64.and
              tee_local 6
              i64.const -1
              i64.add
              i64.const 9218868437227405310
              i64.le_u
              br_if 0 (;@5;)
              get_local 6
              i64.const 9218868437227405313
              i64.lt_u
              br_if 1 (;@4;)
              get_local 6
              i64.const 2251799813685248
              i64.or
              f64.reinterpret/i64
              return
            end
            get_local 7
            i64.const -1
            i64.add
            i64.const 9218868437227405310
            i64.le_u
            br_if 1 (;@3;)
          end
          block  ;; label = @4
            get_local 7
            i64.const 9218868437227405313
            i64.lt_u
            br_if 0 (;@4;)
            get_local 7
            i64.const 2251799813685248
            i64.or
            f64.reinterpret/i64
            return
          end
          block  ;; label = @4
            get_local 6
            i64.const 9218868437227405312
            i64.ne
            br_if 0 (;@4;)
            f64.const nan (;=nan;)
            get_local 0
            get_local 5
            get_local 4
            i64.xor
            i64.const -9223372036854775808
            i64.eq
            select
            return
          end
          get_local 7
          i64.const 9218868437227405312
          i64.eq
          br_if 2 (;@1;)
          get_local 6
          i64.const 0
          i64.eq
          br_if 1 (;@2;)
          get_local 0
          set_local 1
          get_local 7
          i64.eqz
          br_if 2 (;@1;)
        end
        get_local 5
        get_local 4
        get_local 7
        get_local 6
        i64.gt_u
        tee_local 3
        select
        tee_local 7
        i64.const 4503599627370495
        i64.and
        set_local 6
        get_local 4
        get_local 5
        get_local 3
        select
        tee_local 4
        i64.const 52
        i64.shr_u
        i32.wrap/i64
        i32.const 2047
        i32.and
        set_local 2
        block  ;; label = @3
          get_local 7
          i64.const 52
          i64.shr_u
          i32.wrap/i64
          i32.const 2047
          i32.and
          tee_local 3
          br_if 0 (;@3;)
          i32.const 12
          get_local 6
          i64.clz
          tee_local 5
          i32.wrap/i64
          i32.sub
          set_local 3
          get_local 6
          get_local 5
          i64.const 53
          i64.add
          i64.const 63
          i64.and
          i64.shl
          set_local 6
        end
        get_local 4
        i64.const 4503599627370495
        i64.and
        set_local 5
        block  ;; label = @3
          get_local 2
          br_if 0 (;@3;)
          i32.const 12
          get_local 5
          i64.clz
          tee_local 8
          i32.wrap/i64
          i32.sub
          set_local 2
          get_local 5
          get_local 8
          i64.const 53
          i64.add
          i64.const 63
          i64.and
          i64.shl
          set_local 5
        end
        get_local 4
        get_local 7
        i64.xor
        set_local 8
        get_local 5
        i64.const 3
        i64.shl
        i64.const 36028797018963968
        i64.or
        set_local 9
        get_local 6
        i64.const 3
        i64.shl
        set_local 4
        block  ;; label = @3
          block  ;; label = @4
            get_local 3
            get_local 2
            i32.sub
            tee_local 2
            i32.eqz
            br_if 0 (;@4;)
            i64.const 1
            set_local 5
            get_local 2
            i32.const 63
            i32.gt_u
            br_if 1 (;@3;)
            get_local 9
            get_local 2
            i32.const 63
            i32.and
            i64.extend_u/i32
            i64.shr_u
            get_local 9
            i32.const 0
            get_local 2
            i32.sub
            i32.const 63
            i32.and
            i64.extend_u/i32
            i64.shl
            i64.const 0
            i64.ne
            i64.extend_u/i32
            i64.or
            set_local 5
            br 1 (;@3;)
          end
          get_local 9
          set_local 5
        end
        get_local 4
        i64.const 36028797018963968
        i64.or
        set_local 6
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 8
              i64.const -1
              i64.le_s
              br_if 0 (;@5;)
              get_local 5
              get_local 6
              i64.add
              tee_local 6
              i64.const 72057594037927936
              i64.and
              i64.eqz
              br_if 1 (;@4;)
              get_local 5
              get_local 4
              i64.add
              i64.const 1
              i64.and
              get_local 6
              i64.const 1
              i64.shr_u
              i64.or
              set_local 6
              get_local 3
              i32.const 1
              i32.add
              set_local 3
              br 1 (;@4;)
            end
            get_local 6
            get_local 5
            i64.sub
            tee_local 6
            i64.eqz
            br_if 1 (;@3;)
            get_local 6
            i64.const 36028797018963967
            i64.gt_u
            br_if 0 (;@4;)
            get_local 3
            get_local 6
            i64.clz
            i32.wrap/i64
            i32.const -8
            i32.add
            tee_local 2
            i32.sub
            set_local 3
            get_local 6
            get_local 2
            i32.const 63
            i32.and
            i64.extend_u/i32
            i64.shl
            set_local 6
          end
          get_local 7
          i64.const -9223372036854775808
          i64.and
          set_local 7
          block  ;; label = @4
            get_local 3
            i32.const 2047
            i32.lt_s
            br_if 0 (;@4;)
            get_local 7
            i64.const 9218868437227405312
            i64.or
            f64.reinterpret/i64
            return
          end
          i32.const 0
          set_local 2
          block  ;; label = @4
            block  ;; label = @5
              get_local 3
              i32.const 0
              i32.le_s
              br_if 0 (;@5;)
              get_local 3
              set_local 2
              br 1 (;@4;)
            end
            get_local 6
            i32.const 1
            get_local 3
            i32.sub
            tee_local 3
            i32.const 63
            i32.and
            i64.extend_u/i32
            i64.shr_u
            get_local 6
            i32.const 0
            get_local 3
            i32.sub
            i32.const 63
            i32.and
            i64.extend_u/i32
            i64.shl
            i64.const 0
            i64.ne
            i64.extend_u/i32
            i64.or
            set_local 6
          end
          get_local 6
          i64.const 3
          i64.shr_u
          tee_local 5
          i64.const 4503599627370495
          i64.and
          get_local 7
          i64.or
          get_local 2
          i64.extend_u/i32
          i64.const 52
          i64.shl
          i64.or
          set_local 7
          block  ;; label = @4
            block  ;; label = @5
              get_local 6
              i32.wrap/i64
              i32.const 7
              i32.and
              tee_local 3
              i32.const 5
              i32.lt_u
              br_if 0 (;@5;)
              get_local 7
              i64.const 1
              i64.add
              set_local 7
              br 1 (;@4;)
            end
            get_local 3
            i32.const 4
            i32.ne
            br_if 0 (;@4;)
            get_local 7
            get_local 5
            i64.const 1
            i64.and
            i64.add
            set_local 7
          end
          get_local 7
          f64.reinterpret/i64
          set_local 1
          br 2 (;@1;)
        end
        f64.const 0x0p+0 (;=0;)
        return
      end
      get_local 7
      i64.eqz
      i32.eqz
      br_if 0 (;@1;)
      get_local 5
      get_local 4
      i64.and
      f64.reinterpret/i64
      return
    end
    get_local 1)
  (func $__muldi3 (type 5) (param i64 i64) (result i64)
    (local i32 i32 i32 i32 i32 i32)
    get_local 1
    i64.const 32
    i64.shr_u
    i32.wrap/i64
    get_local 0
    i32.wrap/i64
    tee_local 2
    i32.mul
    get_local 1
    i32.wrap/i64
    tee_local 4
    get_local 0
    i64.const 32
    i64.shr_u
    i32.wrap/i64
    i32.mul
    get_local 2
    i32.const 65535
    i32.and
    tee_local 5
    get_local 4
    i32.const 16
    i32.shr_u
    tee_local 6
    i32.mul
    tee_local 7
    get_local 4
    i32.const 65535
    i32.and
    tee_local 4
    get_local 2
    i32.const 16
    i32.shr_u
    tee_local 3
    i32.mul
    get_local 4
    get_local 5
    i32.mul
    tee_local 4
    i32.const 16
    i32.shr_u
    i32.add
    tee_local 2
    i32.const 65535
    i32.and
    i32.add
    i32.const 16
    i32.shr_u
    get_local 2
    i32.const 16
    i32.shr_u
    i32.add
    get_local 6
    get_local 3
    i32.mul
    i32.add
    i32.add
    i32.add
    i64.extend_u/i32
    i64.const 32
    i64.shl
    get_local 7
    get_local 2
    i32.add
    i32.const 16
    i32.shl
    get_local 4
    i32.const 65535
    i32.and
    i32.or
    i64.extend_u/i32
    i64.or)
  (func $__multi3 (type 7) (param i32 i64 i64 i64 i64)
    (local i64 i64 i64 i64 i64)
    get_local 0
    get_local 1
    i64.const 4294967295
    i64.and
    tee_local 7
    get_local 3
    i64.const 32
    i64.shr_u
    tee_local 8
    i64.mul
    tee_local 9
    get_local 3
    i64.const 4294967295
    i64.and
    tee_local 6
    get_local 1
    i64.const 32
    i64.shr_u
    tee_local 5
    i64.mul
    get_local 6
    get_local 7
    i64.mul
    tee_local 6
    i64.const 32
    i64.shr_u
    i64.add
    tee_local 7
    i64.add
    i64.const 32
    i64.shl
    get_local 6
    i64.const 4294967295
    i64.and
    i64.or
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 4
    get_local 1
    i64.mul
    get_local 3
    get_local 2
    i64.mul
    get_local 9
    get_local 7
    i64.const 4294967295
    i64.and
    i64.add
    i64.const 32
    i64.shr_u
    get_local 7
    i64.const 32
    i64.shr_u
    i64.add
    get_local 8
    get_local 5
    i64.mul
    i64.add
    i64.add
    i64.add
    i64.store)
  (func $__mulosi4 (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32 i32 i32)
    get_local 2
    i32.const 0
    i32.store
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 0
          i32.const -2147483648
          i32.ne
          br_if 0 (;@3;)
          get_local 1
          i32.const 2
          i32.lt_u
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i32.mul
          return
        end
        block  ;; label = @3
          get_local 1
          i32.const -2147483648
          i32.ne
          br_if 0 (;@3;)
          get_local 0
          i32.const 2
          i32.lt_u
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i32.mul
          return
        end
        get_local 0
        i32.const 31
        i32.shr_s
        tee_local 3
        get_local 0
        i32.xor
        get_local 3
        i32.sub
        tee_local 4
        i32.const 2
        i32.lt_s
        br_if 0 (;@2;)
        get_local 1
        i32.const 31
        i32.shr_s
        tee_local 5
        get_local 1
        i32.xor
        get_local 5
        i32.sub
        tee_local 6
        i32.const 2
        i32.lt_s
        br_if 0 (;@2;)
        block  ;; label = @3
          get_local 3
          get_local 5
          i32.ne
          br_if 0 (;@3;)
          get_local 6
          i32.eqz
          br_if 2 (;@1;)
          get_local 4
          i32.const 2147483647
          get_local 6
          i32.div_s
          i32.le_s
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i32.mul
          return
        end
        get_local 6
        i32.eqz
        br_if 1 (;@1;)
        i32.const 0
        get_local 6
        i32.sub
        tee_local 3
        i32.const -1
        i32.eq
        br_if 1 (;@1;)
        get_local 4
        i32.const -2147483648
        get_local 3
        i32.div_s
        i32.le_s
        br_if 0 (;@2;)
        get_local 2
        i32.const 1
        i32.store
      end
      get_local 1
      get_local 0
      i32.mul
      return
    end
    unreachable
    unreachable)
  (func $__mulodi4 (type 6) (param i64 i64 i32) (result i64)
    (local i64 i64 i64 i64)
    get_local 2
    i32.const 0
    i32.store
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 0
          i64.const -9223372036854775808
          i64.ne
          br_if 0 (;@3;)
          get_local 1
          i64.const 2
          i64.lt_u
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i64.mul
          return
        end
        block  ;; label = @3
          get_local 1
          i64.const -9223372036854775808
          i64.ne
          br_if 0 (;@3;)
          get_local 0
          i64.const 2
          i64.lt_u
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i64.mul
          return
        end
        get_local 0
        i64.const 63
        i64.shr_s
        tee_local 3
        get_local 0
        i64.xor
        get_local 3
        i64.sub
        tee_local 4
        i64.const 2
        i64.lt_s
        br_if 0 (;@2;)
        get_local 1
        i64.const 63
        i64.shr_s
        tee_local 5
        get_local 1
        i64.xor
        get_local 5
        i64.sub
        tee_local 6
        i64.const 2
        i64.lt_s
        br_if 0 (;@2;)
        block  ;; label = @3
          get_local 3
          get_local 5
          i64.ne
          br_if 0 (;@3;)
          get_local 6
          i64.const 0
          i64.eq
          br_if 2 (;@1;)
          get_local 4
          i64.const 9223372036854775807
          get_local 6
          i64.div_s
          i64.le_s
          br_if 1 (;@2;)
          get_local 2
          i32.const 1
          i32.store
          get_local 1
          get_local 0
          i64.mul
          return
        end
        get_local 6
        i64.eqz
        br_if 1 (;@1;)
        i64.const 0
        get_local 6
        i64.sub
        tee_local 3
        i64.const -1
        i64.eq
        br_if 1 (;@1;)
        get_local 4
        i64.const -9223372036854775808
        get_local 3
        i64.div_s
        i64.le_s
        br_if 0 (;@2;)
        get_local 2
        i32.const 1
        i32.store
      end
      get_local 1
      get_local 0
      i64.mul
      return
    end
    unreachable
    unreachable)
  (func $__muloti4 (type 8) (param i32 i64 i64 i64 i64 i32)
    (local i32 i64 i64 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 6
    i32.store offset=4
    get_local 6
    i32.const 32
    i32.add
    get_local 3
    get_local 4
    get_local 1
    get_local 2
    call $__multi3
    get_local 5
    i32.const 0
    i32.store
    get_local 6
    i32.const 40
    i32.add
    i64.load
    set_local 8
    get_local 6
    i64.load offset=32
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 1
          get_local 2
          i64.const -9223372036854775808
          i64.xor
          i64.or
          i64.const 0
          i64.ne
          br_if 0 (;@3;)
          get_local 3
          i64.const 2
          i64.lt_u
          i32.const 0
          get_local 4
          i64.eqz
          select
          br_if 1 (;@2;)
          get_local 5
          i32.const 1
          i32.store
          br 1 (;@2;)
        end
        block  ;; label = @3
          get_local 3
          get_local 4
          i64.const -9223372036854775808
          i64.xor
          i64.or
          i64.eqz
          i32.eqz
          br_if 0 (;@3;)
          get_local 1
          i64.const 2
          i64.lt_u
          i32.const 0
          get_local 2
          i64.eqz
          select
          br_if 1 (;@2;)
          get_local 5
          i32.const 1
          i32.store
          br 1 (;@2;)
        end
        get_local 2
        i64.const 63
        i64.shr_s
        tee_local 9
        get_local 1
        i64.xor
        tee_local 1
        get_local 9
        i64.sub
        tee_local 10
        i64.const 2
        i64.lt_u
        get_local 9
        get_local 2
        i64.xor
        get_local 9
        i64.sub
        get_local 1
        get_local 9
        i64.lt_u
        i64.extend_u/i32
        i64.sub
        tee_local 1
        i64.const 0
        i64.lt_s
        get_local 1
        i64.eqz
        select
        br_if 0 (;@2;)
        get_local 4
        i64.const 63
        i64.shr_s
        tee_local 2
        get_local 3
        i64.xor
        tee_local 11
        get_local 2
        i64.sub
        tee_local 3
        i64.const 2
        i64.lt_u
        get_local 2
        get_local 4
        i64.xor
        get_local 2
        i64.sub
        get_local 11
        get_local 2
        i64.lt_u
        i64.extend_u/i32
        i64.sub
        tee_local 4
        i64.const 0
        i64.lt_s
        get_local 4
        i64.eqz
        select
        br_if 0 (;@2;)
        block  ;; label = @3
          get_local 9
          get_local 2
          i64.xor
          tee_local 2
          get_local 2
          i64.or
          i64.const 0
          i64.ne
          br_if 0 (;@3;)
          get_local 3
          get_local 4
          i64.or
          i64.const 0
          i64.eq
          br_if 2 (;@1;)
          get_local 6
          i64.const -1
          i64.const 9223372036854775807
          get_local 3
          get_local 4
          call $__divti3
          get_local 10
          get_local 6
          i64.load
          i64.gt_u
          get_local 1
          get_local 6
          i32.const 8
          i32.add
          i64.load
          tee_local 2
          i64.gt_s
          get_local 1
          get_local 2
          i64.eq
          select
          i32.eqz
          br_if 1 (;@2;)
          get_local 5
          i32.const 1
          i32.store
          br 1 (;@2;)
        end
        get_local 3
        get_local 4
        i64.or
        i64.eqz
        br_if 1 (;@1;)
        i64.const 0
        get_local 3
        i64.sub
        tee_local 2
        i64.const 0
        get_local 4
        i64.sub
        get_local 3
        i64.const 0
        i64.ne
        i64.extend_u/i32
        i64.sub
        tee_local 4
        i64.and
        i64.const -1
        i64.eq
        br_if 1 (;@1;)
        get_local 6
        i32.const 16
        i32.add
        i64.const 0
        i64.const -9223372036854775808
        get_local 2
        get_local 4
        call $__divti3
        get_local 10
        get_local 6
        i64.load offset=16
        i64.gt_u
        get_local 1
        get_local 6
        i32.const 24
        i32.add
        i64.load
        tee_local 2
        i64.gt_s
        get_local 1
        get_local 2
        i64.eq
        select
        i32.eqz
        br_if 0 (;@2;)
        get_local 5
        i32.const 1
        i32.store
      end
      get_local 0
      get_local 7
      i64.store
      get_local 0
      i32.const 8
      i32.add
      get_local 8
      i64.store
      i32.const 0
      get_local 6
      i32.const 48
      i32.add
      i32.store offset=4
      return
    end
    unreachable
    unreachable)
  (func $__powisf2 (type 9) (param f32 i32) (result f32)
    (local i32 f32)
    get_local 0
    f32.const 0x1p+0 (;=1;)
    get_local 1
    i32.const 1
    i32.and
    select
    set_local 3
    block  ;; label = @1
      get_local 1
      i32.const 1
      i32.add
      i32.const 3
      i32.lt_u
      br_if 0 (;@1;)
      get_local 1
      set_local 2
      loop  ;; label = @2
        get_local 3
        get_local 0
        get_local 0
        f32.mul
        tee_local 0
        f32.mul
        get_local 3
        get_local 2
        i32.const 2
        i32.div_s
        tee_local 2
        i32.const 1
        i32.and
        select
        set_local 3
        get_local 2
        i32.const 1
        i32.add
        i32.const 2
        i32.gt_u
        br_if 0 (;@2;)
      end
    end
    f32.const 0x1p+0 (;=1;)
    get_local 3
    f32.div
    get_local 3
    get_local 1
    i32.const 0
    i32.lt_s
    select)
  (func $__powidf2 (type 10) (param f64 i32) (result f64)
    (local i32 f64)
    get_local 0
    f64.const 0x1p+0 (;=1;)
    get_local 1
    i32.const 1
    i32.and
    select
    set_local 3
    block  ;; label = @1
      get_local 1
      i32.const 1
      i32.add
      i32.const 3
      i32.lt_u
      br_if 0 (;@1;)
      get_local 1
      set_local 2
      loop  ;; label = @2
        get_local 3
        get_local 0
        get_local 0
        f64.mul
        tee_local 0
        f64.mul
        get_local 3
        get_local 2
        i32.const 2
        i32.div_s
        tee_local 2
        i32.const 1
        i32.and
        select
        set_local 3
        get_local 2
        i32.const 1
        i32.add
        i32.const 2
        i32.gt_u
        br_if 0 (;@2;)
      end
    end
    f64.const 0x1p+0 (;=1;)
    get_local 3
    f64.div
    get_local 3
    get_local 1
    i32.const 0
    i32.lt_s
    select)
  (func $__mulsf3 (type 2) (param f32 f32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64)
    get_local 1
    i32.reinterpret/f32
    tee_local 3
    i32.const 8388607
    i32.and
    set_local 7
    get_local 0
    i32.reinterpret/f32
    tee_local 2
    i32.const 8388607
    i32.and
    set_local 6
    get_local 3
    get_local 2
    i32.xor
    i32.const -2147483648
    i32.and
    set_local 11
    get_local 3
    i32.const 23
    i32.shr_u
    i32.const 255
    i32.and
    set_local 5
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 2
          i32.const 23
          i32.shr_u
          i32.const 255
          i32.and
          tee_local 4
          i32.const -1
          i32.add
          i32.const 253
          i32.gt_u
          br_if 0 (;@3;)
          i32.const 0
          set_local 10
          get_local 5
          i32.const -1
          i32.add
          i32.const 254
          i32.lt_u
          br_if 1 (;@2;)
        end
        block  ;; label = @3
          get_local 2
          i32.const 2147483647
          i32.and
          tee_local 8
          i32.const 2139095041
          i32.lt_u
          br_if 0 (;@3;)
          get_local 2
          i32.const 4194304
          i32.or
          f32.reinterpret/i32
          return
        end
        block  ;; label = @3
          get_local 3
          i32.const 2147483647
          i32.and
          tee_local 9
          i32.const 2139095041
          i32.lt_u
          br_if 0 (;@3;)
          get_local 3
          i32.const 4194304
          i32.or
          f32.reinterpret/i32
          return
        end
        block  ;; label = @3
          get_local 8
          i32.const 2139095040
          i32.ne
          br_if 0 (;@3;)
          get_local 3
          i32.const -2147483648
          i32.and
          get_local 2
          i32.xor
          i32.const 2143289344
          get_local 9
          select
          f32.reinterpret/i32
          return
        end
        block  ;; label = @3
          get_local 9
          i32.const 2139095040
          i32.ne
          br_if 0 (;@3;)
          get_local 2
          i32.const -2147483648
          i32.and
          get_local 3
          i32.xor
          i32.const 2143289344
          get_local 8
          select
          f32.reinterpret/i32
          return
        end
        get_local 8
        i32.eqz
        br_if 1 (;@1;)
        get_local 9
        i32.eqz
        br_if 1 (;@1;)
        i32.const 0
        set_local 10
        block  ;; label = @3
          get_local 8
          i32.const 8388607
          i32.gt_u
          br_if 0 (;@3;)
          i32.const 9
          get_local 6
          i32.clz
          tee_local 2
          i32.sub
          set_local 10
          get_local 6
          get_local 2
          i32.const 24
          i32.add
          i32.const 31
          i32.and
          i32.shl
          set_local 6
        end
        get_local 9
        i32.const 8388607
        i32.gt_u
        br_if 0 (;@2;)
        i32.const 9
        get_local 7
        i32.clz
        tee_local 2
        i32.sub
        get_local 10
        i32.add
        set_local 10
        get_local 7
        get_local 2
        i32.const 24
        i32.add
        i32.const 31
        i32.and
        i32.shl
        set_local 7
      end
      get_local 7
      i32.const 8
      i32.shl
      i32.const -2147483648
      i32.or
      i64.extend_u/i32
      get_local 6
      i32.const 8388608
      i32.or
      i64.extend_u/i32
      i64.mul
      tee_local 12
      i32.wrap/i64
      set_local 2
      get_local 4
      get_local 10
      i32.add
      get_local 5
      i32.add
      set_local 3
      block  ;; label = @2
        block  ;; label = @3
          get_local 12
          i64.const 32
          i64.shr_u
          tee_local 12
          i32.wrap/i64
          tee_local 5
          i32.const 8388608
          i32.and
          br_if 0 (;@3;)
          get_local 2
          i32.const 31
          i32.shr_u
          get_local 12
          i32.wrap/i64
          i32.const 1
          i32.shl
          i32.or
          set_local 5
          get_local 2
          i32.const 1
          i32.shl
          set_local 2
          get_local 3
          i32.const -127
          i32.add
          set_local 3
          br 1 (;@2;)
        end
        get_local 3
        i32.const -126
        i32.add
        set_local 3
      end
      block  ;; label = @2
        get_local 3
        i32.const 255
        i32.lt_s
        br_if 0 (;@2;)
        get_local 11
        i32.const 2139095040
        i32.or
        f32.reinterpret/i32
        return
      end
      block  ;; label = @2
        block  ;; label = @3
          get_local 3
          i32.const 0
          i32.le_s
          br_if 0 (;@3;)
          get_local 5
          i32.const 8388607
          i32.and
          get_local 3
          i32.const 23
          i32.shl
          i32.or
          set_local 3
          br 1 (;@2;)
        end
        i32.const 1
        get_local 3
        i32.sub
        tee_local 3
        i32.const 31
        i32.gt_s
        br_if 1 (;@1;)
        get_local 2
        get_local 3
        i32.rotr
        get_local 5
        i32.const 0
        get_local 3
        i32.sub
        i32.const 31
        i32.and
        i32.shl
        i32.or
        set_local 2
        get_local 5
        get_local 3
        i32.const 31
        i32.and
        i32.shr_u
        set_local 3
      end
      get_local 3
      get_local 11
      i32.or
      set_local 11
      block  ;; label = @2
        get_local 2
        i32.const -2147483647
        i32.lt_u
        br_if 0 (;@2;)
        get_local 11
        i32.const 1
        i32.add
        f32.reinterpret/i32
        return
      end
      get_local 2
      i32.const -2147483648
      i32.ne
      br_if 0 (;@1;)
      get_local 3
      i32.const 1
      i32.and
      get_local 11
      i32.add
      set_local 11
    end
    get_local 11
    f32.reinterpret/i32)
  (func $__muldf3 (type 3) (param f64 f64) (result f64)
    (local i32 i32 i64 i64 i64 i64 i64 i64 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 3
    i32.store offset=4
    get_local 1
    i64.reinterpret/f64
    tee_local 5
    i64.const 4503599627370495
    i64.and
    set_local 9
    get_local 0
    i64.reinterpret/f64
    tee_local 4
    i64.const 4503599627370495
    i64.and
    set_local 8
    get_local 5
    get_local 4
    i64.xor
    i64.const -9223372036854775808
    i64.and
    set_local 12
    get_local 5
    i64.const 52
    i64.shr_u
    i64.const 2047
    i64.and
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 4
          i64.const 52
          i64.shr_u
          i64.const 2047
          i64.and
          tee_local 6
          i64.const -1
          i64.add
          i64.const 2045
          i64.gt_u
          br_if 0 (;@3;)
          i32.const 0
          set_local 2
          get_local 7
          i64.const -1
          i64.add
          i64.const 2046
          i64.lt_u
          br_if 1 (;@2;)
        end
        block  ;; label = @3
          get_local 4
          i64.const 9223372036854775807
          i64.and
          tee_local 10
          i64.const 9218868437227405313
          i64.lt_u
          br_if 0 (;@3;)
          get_local 4
          i64.const 2251799813685248
          i64.or
          set_local 12
          br 2 (;@1;)
        end
        block  ;; label = @3
          get_local 5
          i64.const 9223372036854775807
          i64.and
          tee_local 11
          i64.const 9218868437227405313
          i64.lt_u
          br_if 0 (;@3;)
          get_local 5
          i64.const 2251799813685248
          i64.or
          set_local 12
          br 2 (;@1;)
        end
        block  ;; label = @3
          get_local 10
          i64.const 9218868437227405312
          i64.ne
          br_if 0 (;@3;)
          get_local 5
          i64.const -9223372036854775808
          i64.and
          get_local 4
          i64.xor
          i64.const 9221120237041090560
          get_local 11
          i64.const 0
          i64.ne
          select
          set_local 12
          br 2 (;@1;)
        end
        block  ;; label = @3
          get_local 11
          i64.const 9218868437227405312
          i64.ne
          br_if 0 (;@3;)
          get_local 4
          i64.const -9223372036854775808
          i64.and
          get_local 5
          i64.xor
          i64.const 9221120237041090560
          get_local 10
          i64.const 0
          i64.ne
          select
          set_local 12
          br 2 (;@1;)
        end
        get_local 10
        i64.eqz
        br_if 1 (;@1;)
        get_local 11
        i64.eqz
        br_if 1 (;@1;)
        i32.const 0
        set_local 2
        block  ;; label = @3
          get_local 10
          i64.const 4503599627370495
          i64.gt_u
          br_if 0 (;@3;)
          i32.const 12
          get_local 8
          i64.clz
          tee_local 4
          i32.wrap/i64
          i32.sub
          set_local 2
          get_local 8
          get_local 4
          i64.const 53
          i64.add
          i64.const 63
          i64.and
          i64.shl
          set_local 8
        end
        get_local 11
        i64.const 4503599627370495
        i64.gt_u
        br_if 0 (;@2;)
        i32.const 12
        get_local 9
        i64.clz
        tee_local 4
        i32.wrap/i64
        i32.sub
        get_local 2
        i32.add
        set_local 2
        get_local 9
        get_local 4
        i64.const 53
        i64.add
        i64.const 63
        i64.and
        i64.shl
        set_local 9
      end
      get_local 3
      get_local 9
      i64.const 11
      i64.shl
      i64.const -9223372036854775808
      i64.or
      i64.const 0
      get_local 8
      i64.const 4503599627370496
      i64.or
      i64.const 0
      call $__multi3
      get_local 6
      i32.wrap/i64
      get_local 2
      i32.add
      get_local 7
      i32.wrap/i64
      i32.add
      set_local 2
      get_local 3
      i64.load
      set_local 4
      block  ;; label = @2
        block  ;; label = @3
          get_local 3
          i32.const 8
          i32.add
          i64.load
          tee_local 5
          i64.const 4503599627370496
          i64.and
          i64.eqz
          i32.eqz
          br_if 0 (;@3;)
          get_local 4
          i64.const 63
          i64.shr_u
          get_local 5
          i64.const 1
          i64.shl
          i64.or
          set_local 5
          get_local 4
          i64.const 1
          i64.shl
          set_local 4
          get_local 2
          i32.const -1023
          i32.add
          set_local 2
          br 1 (;@2;)
        end
        get_local 2
        i32.const -1022
        i32.add
        set_local 2
      end
      block  ;; label = @2
        get_local 2
        i32.const 2047
        i32.lt_s
        br_if 0 (;@2;)
        get_local 12
        i64.const 9218868437227405312
        i64.or
        set_local 12
        br 1 (;@1;)
      end
      block  ;; label = @2
        block  ;; label = @3
          get_local 2
          i32.const 0
          i32.le_s
          br_if 0 (;@3;)
          get_local 5
          i64.const 4503599627370495
          i64.and
          get_local 2
          i64.extend_u/i32
          i64.const 52
          i64.shl
          i64.or
          set_local 5
          br 1 (;@2;)
        end
        i32.const 1
        get_local 2
        i32.sub
        tee_local 2
        i32.const 63
        i32.gt_s
        br_if 1 (;@1;)
        get_local 4
        get_local 2
        i32.const 63
        i32.and
        i64.extend_u/i32
        tee_local 7
        i64.rotr
        get_local 5
        i32.const 0
        get_local 2
        i32.sub
        i32.const 63
        i32.and
        i64.extend_u/i32
        i64.shl
        i64.or
        set_local 4
        get_local 5
        get_local 7
        i64.shr_u
        set_local 5
      end
      get_local 5
      get_local 12
      i64.or
      set_local 12
      block  ;; label = @2
        get_local 4
        i64.const -9223372036854775807
        i64.lt_u
        br_if 0 (;@2;)
        get_local 12
        i64.const 1
        i64.add
        set_local 12
        br 1 (;@1;)
      end
      get_local 4
      i64.const -9223372036854775808
      i64.ne
      br_if 0 (;@1;)
      get_local 5
      i64.const 1
      i64.and
      get_local 12
      i64.add
      set_local 12
    end
    i32.const 0
    get_local 3
    i32.const 16
    i32.add
    i32.store offset=4
    get_local 12
    f64.reinterpret/i64)
  (func $__divsi3 (type 4) (param i32 i32) (result i32)
    (local i32 i32)
    block  ;; label = @1
      get_local 1
      i32.const 31
      i32.shr_s
      tee_local 3
      get_local 1
      i32.xor
      get_local 3
      i32.sub
      tee_local 3
      i32.eqz
      br_if 0 (;@1;)
      get_local 0
      i32.const 31
      i32.shr_s
      tee_local 2
      get_local 0
      i32.xor
      get_local 2
      i32.sub
      get_local 3
      i32.div_u
      get_local 1
      get_local 0
      i32.xor
      i32.const 31
      i32.shr_s
      tee_local 1
      i32.xor
      get_local 1
      i32.sub
      return
    end
    unreachable
    unreachable)
  (func $__divdi3 (type 5) (param i64 i64) (result i64)
    (local i64 i64)
    block  ;; label = @1
      get_local 1
      i64.const 63
      i64.shr_s
      tee_local 3
      get_local 1
      i64.xor
      get_local 3
      i64.sub
      tee_local 3
      i64.const 0
      i64.eq
      br_if 0 (;@1;)
      get_local 0
      i64.const 63
      i64.shr_s
      tee_local 2
      get_local 0
      i64.xor
      get_local 2
      i64.sub
      get_local 3
      i64.div_u
      get_local 1
      get_local 0
      i64.xor
      i64.const 63
      i64.shr_s
      tee_local 1
      i64.xor
      get_local 1
      i64.sub
      return
    end
    unreachable
    unreachable)
  (func $__divti3 (type 7) (param i32 i64 i64 i64 i64)
    (local i32 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      get_local 4
      i64.const 63
      i64.shr_s
      tee_local 7
      get_local 3
      i64.xor
      tee_local 3
      get_local 7
      i64.sub
      tee_local 6
      get_local 7
      get_local 4
      i64.xor
      get_local 7
      i64.sub
      get_local 3
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      tee_local 3
      i64.or
      i64.const 0
      i64.eq
      br_if 0 (;@1;)
      get_local 5
      get_local 2
      i64.const 63
      i64.shr_s
      tee_local 7
      get_local 1
      i64.xor
      tee_local 1
      get_local 7
      i64.sub
      get_local 7
      get_local 2
      i64.xor
      get_local 7
      i64.sub
      get_local 1
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      get_local 6
      get_local 3
      call $__udivti3
      get_local 0
      get_local 5
      i64.load
      get_local 4
      get_local 2
      i64.xor
      i64.const 63
      i64.shr_s
      tee_local 7
      i64.xor
      tee_local 4
      get_local 7
      i64.sub
      i64.store
      get_local 0
      i32.const 8
      i32.add
      get_local 5
      i32.const 8
      i32.add
      i64.load
      get_local 7
      i64.xor
      get_local 7
      i64.sub
      get_local 4
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      i64.store
      i32.const 0
      get_local 5
      i32.const 16
      i32.add
      i32.store offset=4
      return
    end
    unreachable
    unreachable)
  (func $__modsi3 (type 4) (param i32 i32) (result i32)
    (local i32)
    block  ;; label = @1
      get_local 1
      i32.const 31
      i32.shr_s
      tee_local 2
      get_local 1
      i32.xor
      get_local 2
      i32.sub
      tee_local 2
      i32.eqz
      br_if 0 (;@1;)
      get_local 0
      i32.const 31
      i32.shr_s
      tee_local 1
      get_local 0
      i32.xor
      get_local 1
      i32.sub
      get_local 2
      i32.rem_u
      get_local 1
      i32.xor
      get_local 1
      i32.sub
      return
    end
    unreachable
    unreachable)
  (func $__moddi3 (type 5) (param i64 i64) (result i64)
    (local i64)
    block  ;; label = @1
      get_local 1
      i64.const 63
      i64.shr_s
      tee_local 2
      get_local 1
      i64.xor
      get_local 2
      i64.sub
      tee_local 2
      i64.const 0
      i64.eq
      br_if 0 (;@1;)
      get_local 0
      i64.const 63
      i64.shr_s
      tee_local 1
      get_local 0
      i64.xor
      get_local 1
      i64.sub
      get_local 2
      i64.rem_u
      get_local 1
      i64.xor
      get_local 1
      i64.sub
      return
    end
    unreachable
    unreachable)
  (func $__modti3 (type 7) (param i32 i64 i64 i64 i64)
    (local i32 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      get_local 4
      i64.const 63
      i64.shr_s
      tee_local 7
      get_local 3
      i64.xor
      tee_local 3
      get_local 7
      i64.sub
      tee_local 6
      get_local 7
      get_local 4
      i64.xor
      get_local 7
      i64.sub
      get_local 3
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      tee_local 4
      i64.or
      i64.const 0
      i64.eq
      br_if 0 (;@1;)
      get_local 5
      get_local 2
      i64.const 63
      i64.shr_s
      tee_local 7
      get_local 1
      i64.xor
      tee_local 3
      get_local 7
      i64.sub
      get_local 7
      get_local 2
      i64.xor
      get_local 7
      i64.sub
      get_local 3
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      get_local 6
      get_local 4
      call $__umodti3
      get_local 0
      get_local 5
      i64.load
      get_local 7
      i64.xor
      tee_local 4
      get_local 7
      i64.sub
      i64.store
      get_local 0
      i32.const 8
      i32.add
      get_local 5
      i32.const 8
      i32.add
      i64.load
      get_local 7
      i64.xor
      get_local 7
      i64.sub
      get_local 4
      get_local 7
      i64.lt_u
      i64.extend_u/i32
      i64.sub
      i64.store
      i32.const 0
      get_local 5
      i32.const 16
      i32.add
      i32.store offset=4
      return
    end
    unreachable
    unreachable)
  (func $__divmodsi4 (type 1) (param i32 i32 i32) (result i32)
    (local i32 i32)
    block  ;; label = @1
      get_local 1
      i32.const 31
      i32.shr_s
      tee_local 4
      get_local 1
      i32.xor
      get_local 4
      i32.sub
      tee_local 4
      i32.eqz
      br_if 0 (;@1;)
      get_local 2
      get_local 0
      get_local 0
      i32.const 31
      i32.shr_s
      tee_local 3
      get_local 0
      i32.xor
      get_local 3
      i32.sub
      get_local 4
      i32.div_u
      get_local 1
      get_local 0
      i32.xor
      i32.const 31
      i32.shr_s
      tee_local 4
      i32.xor
      get_local 4
      i32.sub
      tee_local 4
      get_local 1
      i32.mul
      i32.sub
      i32.store
      get_local 4
      return
    end
    unreachable
    unreachable)
  (func $__divmoddi4 (type 6) (param i64 i64 i32) (result i64)
    (local i64 i64)
    block  ;; label = @1
      get_local 1
      i64.const 63
      i64.shr_s
      tee_local 4
      get_local 1
      i64.xor
      get_local 4
      i64.sub
      tee_local 4
      i64.const 0
      i64.eq
      br_if 0 (;@1;)
      get_local 2
      get_local 0
      get_local 0
      i64.const 63
      i64.shr_s
      tee_local 3
      get_local 0
      i64.xor
      get_local 3
      i64.sub
      get_local 4
      i64.div_u
      get_local 1
      get_local 0
      i64.xor
      i64.const 63
      i64.shr_s
      tee_local 4
      i64.xor
      get_local 4
      i64.sub
      tee_local 4
      get_local 1
      i64.mul
      i64.sub
      i64.store
      get_local 4
      return
    end
    unreachable
    unreachable)
  (func $__divsf3 (type 2) (param f32 f32) (result f32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i64 i64)
    get_local 1
    i32.reinterpret/f32
    tee_local 3
    i32.const 8388607
    i32.and
    set_local 7
    get_local 0
    i32.reinterpret/f32
    tee_local 2
    i32.const 8388607
    i32.and
    set_local 6
    get_local 3
    get_local 2
    i32.xor
    i32.const -2147483648
    i32.and
    set_local 11
    get_local 3
    i32.const 23
    i32.shr_u
    i32.const 255
    i32.and
    set_local 5
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 2
          i32.const 23
          i32.shr_u
          i32.const 255
          i32.and
          tee_local 4
          i32.const -1
          i32.add
          i32.const 253
          i32.gt_u
          br_if 0 (;@3;)
          i32.const 0
          set_local 10
          get_local 5
          i32.const -1
          i32.add
          i32.const 254
          i32.lt_u
          br_if 1 (;@2;)
        end
        block  ;; label = @3
          get_local 2
          i32.const 2147483647
          i32.and
          tee_local 8
          i32.const 2139095041
          i32.lt_u
          br_if 0 (;@3;)
          get_local 2
          i32.const 4194304
          i32.or
          f32.reinterpret/i32
          return
        end
        block  ;; label = @3
          get_local 3
          i32.const 2147483647
          i32.and
          tee_local 9
          i32.const 2139095041
          i32.lt_u
          br_if 0 (;@3;)
          get_local 3
          i32.const 4194304
          i32.or
          f32.reinterpret/i32
          return
        end
        block  ;; label = @3
          get_local 8
          i32.const 2139095040
          i32.ne
          br_if 0 (;@3;)
          i32.const 2143289344
          get_local 3
          i32.const -2147483648
          i32.and
          get_local 2
          i32.xor
          get_local 9
          i32.const 2139095040
          i32.eq
          select
          f32.reinterpret/i32
          return
        end
        get_local 9
        i32.const 2139095040
        i32.eq
        br_if 1 (;@1;)
        block  ;; label = @3
          block  ;; label = @4
            get_local 8
            i32.eqz
            br_if 0 (;@4;)
            get_local 9
            i32.eqz
            br_if 1 (;@3;)
            i32.const 0
            set_local 10
            block  ;; label = @5
              get_local 8
              i32.const 8388607
              i32.gt_u
              br_if 0 (;@5;)
              i32.const 9
              get_local 6
              i32.clz
              tee_local 2
              i32.sub
              set_local 10
              get_local 6
              get_local 2
              i32.const 24
              i32.add
              i32.const 31
              i32.and
              i32.shl
              set_local 6
            end
            get_local 9
            i32.const 8388607
            i32.gt_u
            br_if 2 (;@2;)
            get_local 10
            get_local 7
            i32.clz
            tee_local 2
            i32.add
            i32.const -9
            i32.add
            set_local 10
            get_local 7
            get_local 2
            i32.const 24
            i32.add
            i32.const 31
            i32.and
            i32.shl
            set_local 7
            br 2 (;@2;)
          end
          get_local 11
          i32.const 2143289344
          get_local 9
          select
          f32.reinterpret/i32
          return
        end
        get_local 11
        i32.const 2139095040
        i32.or
        f32.reinterpret/i32
        return
      end
      block  ;; label = @2
        get_local 10
        get_local 4
        i32.add
        i32.const 127
        i32.add
        get_local 5
        i32.sub
        i32.const -1
        i32.const 0
        i64.const 0
        i64.const 0
        i64.const 0
        i32.const 1963258675
        get_local 7
        i32.const 8388608
        i32.or
        tee_local 3
        i32.const 8
        i32.shl
        tee_local 2
        i32.sub
        i64.extend_u/i32
        tee_local 13
        get_local 2
        i64.extend_u/i32
        tee_local 12
        i64.mul
        i64.const 32
        i64.shr_u
        i64.sub
        i64.const 4294967295
        i64.and
        get_local 13
        i64.mul
        i64.const 31
        i64.shr_u
        i64.const 4294967295
        i64.and
        tee_local 13
        get_local 12
        i64.mul
        i64.const 32
        i64.shr_u
        i64.sub
        i64.const 4294967295
        i64.and
        get_local 13
        i64.mul
        i64.const 31
        i64.shr_u
        i64.const 4294967295
        i64.and
        tee_local 13
        get_local 12
        i64.mul
        i64.const 32
        i64.shr_u
        i64.sub
        i64.const 4294967295
        i64.and
        get_local 13
        i64.mul
        i64.const 31
        i64.shr_u
        i64.const 4294967294
        i64.add
        i64.const 4294967295
        i64.and
        get_local 6
        i32.const 8388608
        i32.or
        tee_local 7
        i32.const 1
        i32.shl
        i64.extend_u/i32
        i64.mul
        i64.const 32
        i64.shr_u
        i32.wrap/i64
        tee_local 5
        i32.const 16777216
        i32.lt_u
        tee_local 6
        select
        i32.add
        tee_local 2
        i32.const 255
        i32.lt_s
        br_if 0 (;@2;)
        get_local 11
        i32.const 2139095040
        i32.or
        f32.reinterpret/i32
        return
      end
      get_local 2
      i32.const 1
      i32.lt_s
      br_if 0 (;@1;)
      get_local 7
      i32.const 24
      i32.const 23
      get_local 6
      select
      i32.shl
      get_local 3
      get_local 5
      get_local 5
      i32.const 16777215
      i32.gt_u
      i32.shr_u
      tee_local 5
      i32.mul
      i32.sub
      i32.const 1
      i32.shl
      get_local 3
      i32.gt_u
      get_local 2
      i32.const 23
      i32.shl
      get_local 5
      i32.const 8388607
      i32.and
      i32.or
      i32.add
      get_local 11
      i32.or
      set_local 11
    end
    get_local 11
    f32.reinterpret/i32)
  (func $__divdf3 (type 3) (param f64 f64) (result f64)
    (local i32 i32 i32 i64 i64 i64 i64 i64 i64 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 4
    i32.store offset=4
    get_local 1
    i64.reinterpret/f64
    tee_local 6
    i64.const 4503599627370495
    i64.and
    set_local 10
    get_local 0
    i64.reinterpret/f64
    tee_local 5
    i64.const 4503599627370495
    i64.and
    set_local 9
    get_local 6
    get_local 5
    i64.xor
    i64.const -9223372036854775808
    i64.and
    set_local 13
    get_local 6
    i64.const 52
    i64.shr_u
    i64.const 2047
    i64.and
    set_local 8
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 5
          i64.const 52
          i64.shr_u
          i64.const 2047
          i64.and
          tee_local 7
          i64.const -1
          i64.add
          i64.const 2045
          i64.gt_u
          br_if 0 (;@3;)
          i32.const 0
          set_local 3
          get_local 8
          i64.const -1
          i64.add
          i64.const 2046
          i64.lt_u
          br_if 1 (;@2;)
        end
        block  ;; label = @3
          get_local 5
          i64.const 9223372036854775807
          i64.and
          tee_local 12
          i64.const 9218868437227405313
          i64.lt_u
          br_if 0 (;@3;)
          get_local 5
          i64.const 2251799813685248
          i64.or
          set_local 13
          br 2 (;@1;)
        end
        block  ;; label = @3
          get_local 6
          i64.const 9223372036854775807
          i64.and
          tee_local 11
          i64.const 9218868437227405313
          i64.lt_u
          br_if 0 (;@3;)
          get_local 6
          i64.const 2251799813685248
          i64.or
          set_local 13
          br 2 (;@1;)
        end
        block  ;; label = @3
          get_local 12
          i64.const 9218868437227405312
          i64.ne
          br_if 0 (;@3;)
          i64.const 9221120237041090560
          get_local 6
          i64.const -9223372036854775808
          i64.and
          get_local 5
          i64.xor
          get_local 11
          i64.const 9218868437227405312
          i64.eq
          select
          set_local 13
          br 2 (;@1;)
        end
        get_local 11
        i64.const 9218868437227405312
        i64.eq
        br_if 1 (;@1;)
        block  ;; label = @3
          block  ;; label = @4
            get_local 12
            i64.const 0
            i64.eq
            br_if 0 (;@4;)
            get_local 11
            i64.const 0
            i64.eq
            br_if 1 (;@3;)
            i32.const 0
            set_local 3
            block  ;; label = @5
              get_local 12
              i64.const 4503599627370495
              i64.gt_u
              br_if 0 (;@5;)
              i32.const 12
              get_local 9
              i64.clz
              tee_local 5
              i32.wrap/i64
              i32.sub
              set_local 3
              get_local 9
              get_local 5
              i64.const 53
              i64.add
              i64.const 63
              i64.and
              i64.shl
              set_local 9
            end
            get_local 11
            i64.const 4503599627370495
            i64.gt_u
            br_if 2 (;@2;)
            get_local 3
            get_local 10
            i64.clz
            tee_local 5
            i32.wrap/i64
            i32.add
            i32.const -12
            i32.add
            set_local 3
            get_local 10
            get_local 5
            i64.const 53
            i64.add
            i64.const 63
            i64.and
            i64.shl
            set_local 10
            br 2 (;@2;)
          end
          i64.const 9221120237041090560
          get_local 13
          get_local 11
          i64.eqz
          select
          set_local 13
          br 2 (;@1;)
        end
        get_local 13
        i64.const 9218868437227405312
        i64.or
        set_local 13
        br 1 (;@1;)
      end
      get_local 4
      i64.const 0
      i64.const 0
      i64.const 0
      i64.const 0
      i64.const 1963258675
      get_local 10
      i64.const 4503599627370496
      i64.or
      tee_local 12
      i64.const 21
      i64.shr_u
      tee_local 5
      i64.sub
      i64.const 4294967295
      i64.and
      tee_local 6
      get_local 5
      i64.const 4294967295
      i64.and
      tee_local 5
      i64.mul
      i64.const 32
      i64.shr_u
      i64.sub
      i64.const 4294967295
      i64.and
      get_local 6
      i64.mul
      i64.const 31
      i64.shr_u
      i64.const 4294967295
      i64.and
      tee_local 6
      get_local 5
      i64.mul
      i64.const 32
      i64.shr_u
      i64.sub
      i64.const 4294967295
      i64.and
      get_local 6
      i64.mul
      i64.const 31
      i64.shr_u
      i64.const 4294967295
      i64.and
      tee_local 6
      get_local 5
      i64.mul
      i64.const 32
      i64.shr_u
      i64.sub
      i64.const 4294967295
      i64.and
      get_local 6
      i64.mul
      i64.const 31
      i64.shr_u
      i64.const 4294967295
      i64.add
      i64.const 4294967295
      i64.and
      tee_local 6
      get_local 5
      i64.mul
      get_local 6
      get_local 10
      i64.const 11
      i64.shl
      i64.const 4294965248
      i64.and
      i64.mul
      i64.const 32
      i64.shr_u
      i64.add
      i64.sub
      tee_local 5
      i64.const 32
      i64.shr_u
      get_local 6
      i64.mul
      get_local 5
      i64.const 4294967295
      i64.and
      get_local 6
      i64.mul
      i64.const 32
      i64.shr_u
      i64.add
      i64.const -2
      i64.add
      i64.const 0
      get_local 9
      i64.const 4503599627370496
      i64.or
      tee_local 6
      i64.const 2
      i64.shl
      i64.const 0
      call $__multi3
      block  ;; label = @2
        get_local 3
        get_local 7
        i32.wrap/i64
        i32.add
        i32.const 1023
        i32.add
        get_local 8
        i32.wrap/i64
        i32.sub
        i32.const -1
        i32.const 0
        get_local 4
        i32.const 8
        i32.add
        i64.load
        tee_local 5
        i64.const 9007199254740992
        i64.lt_u
        tee_local 2
        select
        i32.add
        tee_local 3
        i32.const 2047
        i32.lt_s
        br_if 0 (;@2;)
        get_local 13
        i64.const 9218868437227405312
        i64.or
        set_local 13
        br 1 (;@1;)
      end
      get_local 3
      i32.const 1
      i32.lt_s
      br_if 0 (;@1;)
      get_local 6
      i64.const 53
      i64.const 52
      get_local 2
      select
      i64.shl
      get_local 12
      get_local 5
      get_local 5
      i64.const 9007199254740991
      i64.gt_u
      i64.extend_u/i32
      i64.shr_u
      tee_local 5
      i64.mul
      i64.sub
      i64.const 1
      i64.shl
      get_local 12
      i64.gt_u
      i64.extend_u/i32
      get_local 3
      i64.extend_u/i32
      i64.const 52
      i64.shl
      get_local 5
      i64.const 4503599627370495
      i64.and
      i64.or
      i64.add
      get_local 13
      i64.or
      set_local 13
    end
    i32.const 0
    get_local 4
    i32.const 16
    i32.add
    i32.store offset=4
    get_local 13
    f64.reinterpret/i64)
  (func $__ashldi3 (type 11) (param i64 i32) (result i64)
    (local i32)
    block  ;; label = @1
      get_local 1
      i32.const 32
      i32.and
      br_if 0 (;@1;)
      block  ;; label = @2
        get_local 1
        i32.eqz
        br_if 0 (;@2;)
        get_local 0
        i32.wrap/i64
        tee_local 2
        i32.const 0
        get_local 1
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        get_local 0
        i64.const 32
        i64.shr_u
        i32.wrap/i64
        get_local 1
        i32.const 31
        i32.and
        tee_local 1
        i32.shl
        i32.or
        i64.extend_u/i32
        i64.const 32
        i64.shl
        get_local 2
        get_local 1
        i32.shl
        i64.extend_u/i32
        i64.or
        set_local 0
      end
      get_local 0
      return
    end
    get_local 0
    i32.wrap/i64
    get_local 1
    i32.const 31
    i32.and
    i32.shl
    i64.extend_u/i32
    i64.const 32
    i64.shl)
  (func $__ashlti3 (type 12) (param i32 i64 i64 i32)
    (local i64)
    block  ;; label = @1
      block  ;; label = @2
        get_local 3
        i32.const 64
        i32.and
        br_if 0 (;@2;)
        get_local 3
        i32.eqz
        br_if 1 (;@1;)
        get_local 1
        i32.const 0
        get_local 3
        i32.sub
        i32.const 63
        i32.and
        i64.extend_u/i32
        i64.shr_u
        get_local 2
        get_local 3
        i32.const 63
        i32.and
        i64.extend_u/i32
        tee_local 4
        i64.shl
        i64.or
        set_local 2
        get_local 1
        get_local 4
        i64.shl
        set_local 1
        br 1 (;@1;)
      end
      get_local 1
      get_local 3
      i32.const 63
      i32.and
      i64.extend_u/i32
      i64.shl
      set_local 2
      i64.const 0
      set_local 1
    end
    get_local 0
    get_local 1
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 2
    i64.store)
  (func $__ashrdi3 (type 11) (param i64 i32) (result i64)
    (local i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 1
          i32.const 32
          i32.and
          br_if 0 (;@3;)
          get_local 1
          i32.eqz
          br_if 2 (;@1;)
          get_local 0
          i32.wrap/i64
          get_local 1
          i32.const 31
          i32.and
          tee_local 3
          i32.shr_u
          get_local 0
          i64.const 32
          i64.shr_u
          i32.wrap/i64
          tee_local 2
          i32.const 0
          get_local 1
          i32.sub
          i32.const 31
          i32.and
          i32.shl
          i32.or
          set_local 1
          br 1 (;@2;)
        end
        i32.const 31
        set_local 3
        get_local 0
        i64.const 32
        i64.shr_u
        i32.wrap/i64
        tee_local 2
        get_local 1
        i32.const 31
        i32.and
        i32.shr_s
        set_local 1
      end
      get_local 2
      get_local 3
      i32.shr_s
      i64.extend_u/i32
      i64.const 32
      i64.shl
      get_local 1
      i64.extend_u/i32
      i64.or
      set_local 0
    end
    get_local 0)
  (func $__ashrti3 (type 12) (param i32 i64 i64 i32)
    (local i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 3
          i32.const 64
          i32.and
          br_if 0 (;@3;)
          get_local 3
          i32.eqz
          br_if 2 (;@1;)
          get_local 1
          get_local 3
          i32.const 63
          i32.and
          i64.extend_u/i32
          tee_local 4
          i64.shr_u
          get_local 2
          i32.const 0
          get_local 3
          i32.sub
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shl
          i64.or
          set_local 1
          br 1 (;@2;)
        end
        get_local 2
        get_local 3
        i32.const 63
        i32.and
        i64.extend_u/i32
        i64.shr_s
        set_local 1
        i64.const 63
        set_local 4
      end
      get_local 2
      get_local 4
      i64.shr_s
      set_local 2
    end
    get_local 0
    get_local 1
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 2
    i64.store)
  (func $__lshrdi3 (type 11) (param i64 i32) (result i64)
    (local i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 1
          i32.const 32
          i32.and
          br_if 0 (;@3;)
          get_local 1
          i32.eqz
          br_if 2 (;@1;)
          get_local 0
          i32.wrap/i64
          get_local 1
          i32.const 31
          i32.and
          tee_local 3
          i32.shr_u
          get_local 0
          i64.const 32
          i64.shr_u
          i32.wrap/i64
          tee_local 2
          i32.const 0
          get_local 1
          i32.sub
          i32.const 31
          i32.and
          i32.shl
          i32.or
          set_local 1
          get_local 2
          get_local 3
          i32.shr_u
          i64.extend_u/i32
          i64.const 32
          i64.shl
          set_local 0
          br 1 (;@2;)
        end
        get_local 0
        i64.const 32
        i64.shr_u
        i32.wrap/i64
        get_local 1
        i32.const 31
        i32.and
        i32.shr_u
        set_local 1
        i64.const 0
        set_local 0
      end
      get_local 0
      get_local 1
      i64.extend_u/i32
      i64.or
      set_local 0
    end
    get_local 0)
  (func $__lshrti3 (type 12) (param i32 i64 i64 i32)
    (local i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 3
          i32.const 64
          i32.and
          br_if 0 (;@3;)
          get_local 3
          i32.eqz
          br_if 2 (;@1;)
          get_local 1
          get_local 3
          i32.const 63
          i32.and
          i64.extend_u/i32
          tee_local 4
          i64.shr_u
          get_local 2
          i32.const 0
          get_local 3
          i32.sub
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shl
          i64.or
          set_local 1
          get_local 2
          get_local 4
          i64.shr_u
          set_local 2
          i64.const 0
          set_local 4
          br 1 (;@2;)
        end
        get_local 2
        get_local 3
        i32.const 63
        i32.and
        i64.extend_u/i32
        i64.shr_u
        set_local 1
        i64.const 0
        set_local 4
        i64.const 0
        set_local 2
      end
      get_local 4
      get_local 1
      i64.or
      set_local 1
    end
    get_local 0
    get_local 1
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 2
    i64.store)
  (func $__floatsisf (type 13) (param i32) (result f32)
    (local i32 i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i32.eqz
              br_if 0 (;@5;)
              i32.const 31
              get_local 0
              get_local 0
              i32.const 31
              i32.shr_s
              tee_local 1
              i32.add
              get_local 1
              i32.xor
              tee_local 1
              i32.clz
              tee_local 2
              i32.sub
              set_local 4
              i32.const 32
              get_local 2
              i32.sub
              tee_local 2
              i32.const 24
              i32.le_u
              br_if 1 (;@4;)
              get_local 2
              i32.const 25
              i32.eq
              br_if 2 (;@3;)
              get_local 2
              i32.const 26
              i32.eq
              br_if 3 (;@2;)
              get_local 1
              i32.const 58
              get_local 2
              i32.sub
              i32.const 31
              i32.and
              i32.shl
              i32.const 0
              i32.ne
              get_local 1
              get_local 2
              i32.const 6
              i32.add
              i32.const 31
              i32.and
              i32.shr_u
              i32.or
              set_local 1
              br 3 (;@2;)
            end
            f32.const 0x0p+0 (;=0;)
            return
          end
          get_local 1
          i32.const 24
          get_local 2
          i32.sub
          i32.const 31
          i32.and
          i32.shl
          set_local 1
          br 2 (;@1;)
        end
        get_local 1
        i32.const 1
        i32.shl
        set_local 1
      end
      get_local 1
      i32.const 2
      i32.shr_u
      i32.const 1
      i32.and
      get_local 1
      i32.or
      i32.const 1
      i32.add
      tee_local 1
      i32.const 3
      i32.shr_u
      get_local 1
      i32.const 2
      i32.shr_u
      tee_local 1
      get_local 1
      i32.const 16777216
      i32.and
      tee_local 3
      select
      set_local 1
      get_local 2
      get_local 4
      get_local 3
      select
      set_local 4
    end
    get_local 4
    i32.const 23
    i32.shl
    i32.const 1065353216
    i32.add
    i32.const 2139095040
    i32.and
    get_local 0
    i32.const -2147483648
    i32.and
    i32.or
    get_local 1
    i32.const 8388607
    i32.and
    i32.or
    f32.reinterpret/i32)
  (func $__floatsidf (type 14) (param i32) (result f64)
    (local i32 i32)
    block  ;; label = @1
      get_local 0
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1054
      get_local 0
      get_local 0
      i32.const 31
      i32.shr_s
      tee_local 1
      i32.add
      get_local 1
      i32.xor
      tee_local 1
      i32.clz
      tee_local 2
      i32.sub
      i64.extend_u/i32
      i64.const 52
      i64.shl
      get_local 0
      i32.const 31
      i32.shr_u
      i64.extend_u/i32
      i64.const 63
      i64.shl
      i64.or
      get_local 1
      i64.extend_u/i32
      i32.const 117
      i32.const 32
      get_local 2
      i32.sub
      i32.sub
      i32.const 63
      i32.and
      i64.extend_u/i32
      i64.shl
      i64.const 4503599627370495
      i64.and
      i64.or
      f64.reinterpret/i64
      return
    end
    f64.const 0x0p+0 (;=0;))
  (func $__floatdidf (type 15) (param i64) (result f64)
    (local i32 i32 i32 i64 i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i64.eqz
              br_if 0 (;@5;)
              i32.const 63
              get_local 0
              get_local 0
              i64.const 63
              i64.shr_s
              tee_local 4
              i64.add
              get_local 4
              i64.xor
              tee_local 4
              i64.clz
              i32.wrap/i64
              tee_local 1
              i32.sub
              set_local 3
              i32.const 64
              get_local 1
              i32.sub
              tee_local 1
              i32.const 53
              i32.le_u
              br_if 1 (;@4;)
              get_local 1
              i32.const 54
              i32.eq
              br_if 2 (;@3;)
              get_local 1
              i32.const 55
              i32.eq
              br_if 3 (;@2;)
              get_local 4
              i32.const 119
              get_local 1
              i32.sub
              i32.const 63
              i32.and
              i64.extend_u/i32
              i64.shl
              i64.const 0
              i64.ne
              i64.extend_u/i32
              get_local 4
              get_local 1
              i32.const 9
              i32.add
              i32.const 63
              i32.and
              i64.extend_u/i32
              i64.shr_u
              i64.or
              set_local 4
              br 3 (;@2;)
            end
            f64.const 0x0p+0 (;=0;)
            return
          end
          get_local 4
          i32.const 53
          get_local 1
          i32.sub
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shl
          set_local 4
          br 2 (;@1;)
        end
        get_local 4
        i64.const 1
        i64.shl
        set_local 4
      end
      get_local 4
      i64.const 2
      i64.shr_u
      i64.const 1
      i64.and
      get_local 4
      i64.or
      i64.const 1
      i64.add
      tee_local 4
      i64.const 2
      i64.shr_u
      tee_local 5
      get_local 4
      i64.const 3
      i64.shr_u
      get_local 5
      i64.const 9007199254740992
      i64.and
      i64.eqz
      tee_local 2
      select
      set_local 4
      get_local 3
      get_local 1
      get_local 2
      select
      set_local 3
    end
    get_local 3
    i32.const 1023
    i32.add
    i64.extend_u/i32
    i64.const 52
    i64.shl
    i64.const 9218868437227405312
    i64.and
    get_local 0
    i64.const -9223372036854775808
    i64.and
    i64.or
    get_local 4
    i64.const 4503599627370495
    i64.and
    i64.or
    f64.reinterpret/i64)
  (func $__floattisf (type 16) (param i64 i64) (result f32)
    (local i32 i32 i32 i32 i64 i64 f32)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                get_local 0
                get_local 1
                i64.or
                i64.eqz
                br_if 0 (;@6;)
                i32.const 127
                get_local 1
                get_local 1
                i64.const 63
                i64.shr_s
                tee_local 6
                i64.add
                i64.const 1
                get_local 0
                get_local 6
                i64.add
                tee_local 7
                get_local 0
                i64.lt_u
                i64.extend_u/i32
                get_local 7
                get_local 6
                i64.lt_u
                select
                i64.add
                get_local 6
                i64.xor
                tee_local 0
                i64.clz
                get_local 7
                get_local 6
                i64.xor
                tee_local 6
                i64.clz
                i64.const 64
                i64.add
                get_local 0
                i64.const 0
                i64.ne
                select
                i32.wrap/i64
                tee_local 2
                i32.sub
                set_local 4
                i32.const 128
                get_local 2
                i32.sub
                tee_local 2
                i32.const 24
                i32.le_u
                br_if 1 (;@5;)
                get_local 2
                i32.const 25
                i32.eq
                br_if 2 (;@4;)
                get_local 2
                i32.const 26
                i32.eq
                br_if 3 (;@3;)
                get_local 5
                get_local 6
                get_local 0
                i32.const 154
                get_local 2
                i32.sub
                i32.const 127
                i32.and
                call $__ashlti3
                get_local 5
                i32.const 16
                i32.add
                get_local 6
                get_local 0
                get_local 2
                i32.const 102
                i32.add
                i32.const 127
                i32.and
                call $__lshrti3
                get_local 5
                i64.load
                get_local 5
                i32.const 8
                i32.add
                i64.load
                i64.or
                i64.const 0
                i64.ne
                i64.extend_u/i32
                get_local 5
                i64.load offset=16
                i64.or
                set_local 6
                get_local 5
                i32.const 16
                i32.add
                i32.const 8
                i32.add
                i64.load
                set_local 0
                br 3 (;@3;)
              end
              f32.const 0x0p+0 (;=0;)
              set_local 8
              br 4 (;@1;)
            end
            get_local 5
            i32.const 32
            i32.add
            get_local 6
            get_local 0
            i32.const 24
            get_local 2
            i32.sub
            i32.const 127
            i32.and
            call $__ashlti3
            get_local 5
            i64.load offset=32
            set_local 0
            br 2 (;@2;)
          end
          get_local 0
          i64.const 1
          i64.shl
          get_local 6
          i64.const 63
          i64.shr_u
          i64.or
          set_local 0
          get_local 6
          i64.const 1
          i64.shl
          set_local 6
        end
        get_local 6
        i64.const 2
        i64.shr_u
        i64.const 1
        i64.and
        get_local 6
        i64.or
        tee_local 7
        i64.const 1
        i64.add
        tee_local 6
        i64.const 2
        i64.shr_u
        get_local 0
        i64.const 1
        get_local 6
        get_local 7
        i64.lt_u
        i64.extend_u/i32
        get_local 6
        i64.eqz
        select
        i64.add
        tee_local 0
        i64.const 62
        i64.shl
        i64.or
        tee_local 7
        get_local 6
        i64.const 3
        i64.shr_u
        get_local 0
        i64.const 61
        i64.shl
        i64.or
        get_local 7
        i64.const 16777216
        i64.and
        i64.eqz
        tee_local 3
        select
        set_local 0
        get_local 4
        get_local 2
        get_local 3
        select
        set_local 4
      end
      get_local 4
      i32.const 23
      i32.shl
      i32.const 1065353216
      i32.add
      i32.const 2139095040
      i32.and
      get_local 1
      i64.const 32
      i64.shr_u
      i32.wrap/i64
      i32.const -2147483648
      i32.and
      i32.or
      get_local 0
      i32.wrap/i64
      i32.const 8388607
      i32.and
      i32.or
      f32.reinterpret/i32
      set_local 8
    end
    i32.const 0
    get_local 5
    i32.const 48
    i32.add
    i32.store offset=4
    get_local 8)
  (func $__floattidf (type 17) (param i64 i64) (result f64)
    (local i32 i32 i32 i32 i64 i64 f64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                get_local 0
                get_local 1
                i64.or
                i64.eqz
                br_if 0 (;@6;)
                i32.const 127
                get_local 1
                get_local 1
                i64.const 63
                i64.shr_s
                tee_local 6
                i64.add
                i64.const 1
                get_local 0
                get_local 6
                i64.add
                tee_local 7
                get_local 0
                i64.lt_u
                i64.extend_u/i32
                get_local 7
                get_local 6
                i64.lt_u
                select
                i64.add
                get_local 6
                i64.xor
                tee_local 0
                i64.clz
                get_local 7
                get_local 6
                i64.xor
                tee_local 6
                i64.clz
                i64.const 64
                i64.add
                get_local 0
                i64.const 0
                i64.ne
                select
                i32.wrap/i64
                tee_local 2
                i32.sub
                set_local 4
                i32.const 128
                get_local 2
                i32.sub
                tee_local 2
                i32.const 53
                i32.le_u
                br_if 1 (;@5;)
                get_local 2
                i32.const 54
                i32.eq
                br_if 2 (;@4;)
                get_local 2
                i32.const 55
                i32.eq
                br_if 3 (;@3;)
                get_local 5
                get_local 6
                get_local 0
                i32.const 183
                get_local 2
                i32.sub
                i32.const 127
                i32.and
                call $__ashlti3
                get_local 5
                i32.const 16
                i32.add
                get_local 6
                get_local 0
                get_local 2
                i32.const 73
                i32.add
                i32.const 127
                i32.and
                call $__lshrti3
                get_local 5
                i64.load
                get_local 5
                i32.const 8
                i32.add
                i64.load
                i64.or
                i64.const 0
                i64.ne
                i64.extend_u/i32
                get_local 5
                i64.load offset=16
                i64.or
                set_local 6
                get_local 5
                i32.const 16
                i32.add
                i32.const 8
                i32.add
                i64.load
                set_local 0
                br 3 (;@3;)
              end
              f64.const 0x0p+0 (;=0;)
              set_local 8
              br 4 (;@1;)
            end
            get_local 5
            i32.const 32
            i32.add
            get_local 6
            get_local 0
            i32.const 53
            get_local 2
            i32.sub
            i32.const 127
            i32.and
            call $__ashlti3
            get_local 5
            i64.load offset=32
            set_local 0
            br 2 (;@2;)
          end
          get_local 0
          i64.const 1
          i64.shl
          get_local 6
          i64.const 63
          i64.shr_u
          i64.or
          set_local 0
          get_local 6
          i64.const 1
          i64.shl
          set_local 6
        end
        get_local 6
        i64.const 2
        i64.shr_u
        i64.const 1
        i64.and
        get_local 6
        i64.or
        tee_local 7
        i64.const 1
        i64.add
        tee_local 6
        i64.const 2
        i64.shr_u
        get_local 0
        i64.const 1
        get_local 6
        get_local 7
        i64.lt_u
        i64.extend_u/i32
        get_local 6
        i64.eqz
        select
        i64.add
        tee_local 0
        i64.const 62
        i64.shl
        i64.or
        tee_local 7
        get_local 6
        i64.const 3
        i64.shr_u
        get_local 0
        i64.const 61
        i64.shl
        i64.or
        get_local 7
        i64.const 9007199254740992
        i64.and
        i64.eqz
        tee_local 3
        select
        set_local 0
        get_local 4
        get_local 2
        get_local 3
        select
        set_local 4
      end
      get_local 4
      i32.const 1023
      i32.add
      i64.extend_u/i32
      i64.const 52
      i64.shl
      i64.const 9218868437227405312
      i64.and
      get_local 1
      i64.const -9223372036854775808
      i64.and
      i64.or
      get_local 0
      i64.const 4503599627370495
      i64.and
      i64.or
      f64.reinterpret/i64
      set_local 8
    end
    i32.const 0
    get_local 5
    i32.const 48
    i32.add
    i32.store offset=4
    get_local 8)
  (func $__floatunsisf (type 13) (param i32) (result f32)
    (local i32 i32 i32)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i32.eqz
              br_if 0 (;@5;)
              i32.const 31
              get_local 0
              i32.clz
              tee_local 1
              i32.sub
              set_local 3
              i32.const 32
              get_local 1
              i32.sub
              tee_local 1
              i32.const 24
              i32.le_u
              br_if 1 (;@4;)
              get_local 1
              i32.const 25
              i32.eq
              br_if 2 (;@3;)
              get_local 1
              i32.const 26
              i32.eq
              br_if 3 (;@2;)
              get_local 0
              i32.const 58
              get_local 1
              i32.sub
              i32.const 31
              i32.and
              i32.shl
              i32.const 0
              i32.ne
              get_local 0
              get_local 1
              i32.const 6
              i32.add
              i32.const 31
              i32.and
              i32.shr_u
              i32.or
              set_local 0
              br 3 (;@2;)
            end
            f32.const 0x0p+0 (;=0;)
            return
          end
          get_local 0
          i32.const 24
          get_local 1
          i32.sub
          i32.const 31
          i32.and
          i32.shl
          set_local 0
          br 2 (;@1;)
        end
        get_local 0
        i32.const 1
        i32.shl
        set_local 0
      end
      get_local 0
      i32.const 2
      i32.shr_u
      i32.const 1
      i32.and
      get_local 0
      i32.or
      i32.const 1
      i32.add
      tee_local 0
      i32.const 3
      i32.shr_u
      get_local 0
      i32.const 2
      i32.shr_u
      tee_local 0
      get_local 0
      i32.const 16777216
      i32.and
      tee_local 2
      select
      set_local 0
      get_local 1
      get_local 3
      get_local 2
      select
      set_local 3
    end
    get_local 3
    i32.const 23
    i32.shl
    i32.const 1065353216
    i32.add
    i32.const 2139095040
    i32.and
    get_local 0
    i32.const 8388607
    i32.and
    i32.or
    f32.reinterpret/i32)
  (func $__floatunsidf (type 14) (param i32) (result f64)
    (local i32)
    block  ;; label = @1
      get_local 0
      i32.eqz
      br_if 0 (;@1;)
      i32.const 1054
      get_local 0
      i32.clz
      tee_local 1
      i32.sub
      i64.extend_u/i32
      i64.const 52
      i64.shl
      get_local 0
      i64.extend_u/i32
      i32.const 117
      i32.const 32
      get_local 1
      i32.sub
      i32.sub
      i32.const 63
      i32.and
      i64.extend_u/i32
      i64.shl
      i64.const 4503599627370495
      i64.and
      i64.or
      f64.reinterpret/i64
      return
    end
    f64.const 0x0p+0 (;=0;))
  (func $__floatundidf (type 15) (param i64) (result f64)
    (local i32 i32 i32 i64)
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              get_local 0
              i64.eqz
              br_if 0 (;@5;)
              i32.const 63
              get_local 0
              i64.clz
              i32.wrap/i64
              tee_local 1
              i32.sub
              set_local 3
              i32.const 64
              get_local 1
              i32.sub
              tee_local 1
              i32.const 53
              i32.le_u
              br_if 1 (;@4;)
              get_local 1
              i32.const 54
              i32.eq
              br_if 2 (;@3;)
              get_local 1
              i32.const 55
              i32.eq
              br_if 3 (;@2;)
              get_local 0
              i32.const 119
              get_local 1
              i32.sub
              i32.const 63
              i32.and
              i64.extend_u/i32
              i64.shl
              i64.const 0
              i64.ne
              i64.extend_u/i32
              get_local 0
              get_local 1
              i32.const 9
              i32.add
              i32.const 63
              i32.and
              i64.extend_u/i32
              i64.shr_u
              i64.or
              set_local 0
              br 3 (;@2;)
            end
            f64.const 0x0p+0 (;=0;)
            return
          end
          get_local 0
          i32.const 53
          get_local 1
          i32.sub
          i32.const 63
          i32.and
          i64.extend_u/i32
          i64.shl
          set_local 0
          br 2 (;@1;)
        end
        get_local 0
        i64.const 1
        i64.shl
        set_local 0
      end
      get_local 0
      i64.const 2
      i64.shr_u
      i64.const 1
      i64.and
      get_local 0
      i64.or
      i64.const 1
      i64.add
      tee_local 0
      i64.const 2
      i64.shr_u
      tee_local 4
      get_local 0
      i64.const 3
      i64.shr_u
      get_local 4
      i64.const 9007199254740992
      i64.and
      i64.eqz
      tee_local 2
      select
      set_local 0
      get_local 3
      get_local 1
      get_local 2
      select
      set_local 3
    end
    get_local 3
    i32.const 1023
    i32.add
    i64.extend_u/i32
    i64.const 52
    i64.shl
    i64.const 9218868437227405312
    i64.and
    get_local 0
    i64.const 4503599627370495
    i64.and
    i64.or
    f64.reinterpret/i64)
  (func $__floatuntisf (type 16) (param i64 i64) (result f32)
    (local i32 i32 i32 i32 i64 f32)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                get_local 0
                get_local 1
                i64.or
                i64.eqz
                br_if 0 (;@6;)
                i32.const 127
                get_local 1
                i64.clz
                get_local 0
                i64.clz
                i64.const 64
                i64.add
                get_local 1
                i64.const 0
                i64.ne
                select
                i32.wrap/i64
                tee_local 2
                i32.sub
                set_local 4
                i32.const 128
                get_local 2
                i32.sub
                tee_local 2
                i32.const 24
                i32.le_u
                br_if 1 (;@5;)
                get_local 2
                i32.const 25
                i32.eq
                br_if 2 (;@4;)
                get_local 2
                i32.const 26
                i32.eq
                br_if 3 (;@3;)
                get_local 5
                get_local 0
                get_local 1
                i32.const 154
                get_local 2
                i32.sub
                i32.const 127
                i32.and
                call $__ashlti3
                get_local 5
                i32.const 16
                i32.add
                get_local 0
                get_local 1
                get_local 2
                i32.const 102
                i32.add
                i32.const 127
                i32.and
                call $__lshrti3
                get_local 5
                i64.load
                get_local 5
                i32.const 8
                i32.add
                i64.load
                i64.or
                i64.const 0
                i64.ne
                i64.extend_u/i32
                get_local 5
                i64.load offset=16
                i64.or
                set_local 0
                get_local 5
                i32.const 16
                i32.add
                i32.const 8
                i32.add
                i64.load
                set_local 1
                br 3 (;@3;)
              end
              f32.const 0x0p+0 (;=0;)
              set_local 7
              br 4 (;@1;)
            end
            get_local 5
            i32.const 32
            i32.add
            get_local 0
            get_local 1
            i32.const 24
            get_local 2
            i32.sub
            i32.const 127
            i32.and
            call $__ashlti3
            get_local 5
            i64.load offset=32
            set_local 1
            br 2 (;@2;)
          end
          get_local 1
          i64.const 1
          i64.shl
          get_local 0
          i64.const 63
          i64.shr_u
          i64.or
          set_local 1
          get_local 0
          i64.const 1
          i64.shl
          set_local 0
        end
        get_local 0
        i64.const 2
        i64.shr_u
        i64.const 1
        i64.and
        get_local 0
        i64.or
        tee_local 6
        i64.const 1
        i64.add
        tee_local 0
        i64.const 2
        i64.shr_u
        get_local 1
        i64.const 1
        get_local 0
        get_local 6
        i64.lt_u
        i64.extend_u/i32
        get_local 0
        i64.eqz
        select
        i64.add
        tee_local 1
        i64.const 62
        i64.shl
        i64.or
        tee_local 6
        get_local 0
        i64.const 3
        i64.shr_u
        get_local 1
        i64.const 61
        i64.shl
        i64.or
        get_local 6
        i64.const 16777216
        i64.and
        i64.eqz
        tee_local 3
        select
        set_local 1
        get_local 4
        get_local 2
        get_local 3
        select
        set_local 4
      end
      get_local 4
      i32.const 23
      i32.shl
      i32.const 1065353216
      i32.add
      i32.const 2139095040
      i32.and
      get_local 1
      i32.wrap/i64
      i32.const 8388607
      i32.and
      i32.or
      f32.reinterpret/i32
      set_local 7
    end
    i32.const 0
    get_local 5
    i32.const 48
    i32.add
    i32.store offset=4
    get_local 7)
  (func $__floatuntidf (type 17) (param i64 i64) (result f64)
    (local i32 i32 i32 i32 i64 f64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 48
    i32.sub
    tee_local 5
    i32.store offset=4
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                get_local 0
                get_local 1
                i64.or
                i64.eqz
                br_if 0 (;@6;)
                i32.const 127
                get_local 1
                i64.clz
                get_local 0
                i64.clz
                i64.const 64
                i64.add
                get_local 1
                i64.const 0
                i64.ne
                select
                i32.wrap/i64
                tee_local 2
                i32.sub
                set_local 4
                i32.const 128
                get_local 2
                i32.sub
                tee_local 2
                i32.const 53
                i32.le_u
                br_if 1 (;@5;)
                get_local 2
                i32.const 54
                i32.eq
                br_if 2 (;@4;)
                get_local 2
                i32.const 55
                i32.eq
                br_if 3 (;@3;)
                get_local 5
                get_local 0
                get_local 1
                i32.const 183
                get_local 2
                i32.sub
                i32.const 127
                i32.and
                call $__ashlti3
                get_local 5
                i32.const 16
                i32.add
                get_local 0
                get_local 1
                get_local 2
                i32.const 73
                i32.add
                i32.const 127
                i32.and
                call $__lshrti3
                get_local 5
                i64.load
                get_local 5
                i32.const 8
                i32.add
                i64.load
                i64.or
                i64.const 0
                i64.ne
                i64.extend_u/i32
                get_local 5
                i64.load offset=16
                i64.or
                set_local 0
                get_local 5
                i32.const 16
                i32.add
                i32.const 8
                i32.add
                i64.load
                set_local 1
                br 3 (;@3;)
              end
              f64.const 0x0p+0 (;=0;)
              set_local 7
              br 4 (;@1;)
            end
            get_local 5
            i32.const 32
            i32.add
            get_local 0
            get_local 1
            i32.const 53
            get_local 2
            i32.sub
            i32.const 127
            i32.and
            call $__ashlti3
            get_local 5
            i64.load offset=32
            set_local 1
            br 2 (;@2;)
          end
          get_local 1
          i64.const 1
          i64.shl
          get_local 0
          i64.const 63
          i64.shr_u
          i64.or
          set_local 1
          get_local 0
          i64.const 1
          i64.shl
          set_local 0
        end
        get_local 0
        i64.const 2
        i64.shr_u
        i64.const 1
        i64.and
        get_local 0
        i64.or
        tee_local 6
        i64.const 1
        i64.add
        tee_local 0
        i64.const 2
        i64.shr_u
        get_local 1
        i64.const 1
        get_local 0
        get_local 6
        i64.lt_u
        i64.extend_u/i32
        get_local 0
        i64.eqz
        select
        i64.add
        tee_local 1
        i64.const 62
        i64.shl
        i64.or
        tee_local 6
        get_local 0
        i64.const 3
        i64.shr_u
        get_local 1
        i64.const 61
        i64.shl
        i64.or
        get_local 6
        i64.const 9007199254740992
        i64.and
        i64.eqz
        tee_local 3
        select
        set_local 1
        get_local 4
        get_local 2
        get_local 3
        select
        set_local 4
      end
      get_local 4
      i32.const 1023
      i32.add
      i64.extend_u/i32
      i64.const 52
      i64.shl
      i64.const 9218868437227405312
      i64.and
      get_local 1
      i64.const 4503599627370495
      i64.and
      i64.or
      f64.reinterpret/i64
      set_local 7
    end
    i32.const 0
    get_local 5
    i32.const 48
    i32.add
    i32.store offset=4
    get_local 7)
  (func $__fixsfsi (type 18) (param f32) (result i32)
    (local i32 i32 i32 i32)
    i32.const 0
    set_local 4
    block  ;; label = @1
      block  ;; label = @2
        get_local 0
        i32.reinterpret/f32
        tee_local 1
        i32.const 2139095040
        i32.and
        tee_local 3
        i32.const 1065353216
        i32.lt_u
        br_if 0 (;@2;)
        get_local 3
        i32.const 23
        i32.shr_u
        tee_local 4
        i32.const -127
        i32.add
        tee_local 3
        i32.const 30
        i32.le_u
        br_if 1 (;@1;)
        get_local 1
        i32.const 31
        i32.shr_u
        i32.const 2147483647
        i32.add
        set_local 4
      end
      get_local 4
      return
    end
    get_local 1
    i32.const 8388607
    i32.and
    i32.const 8388608
    i32.or
    set_local 2
    block  ;; label = @1
      block  ;; label = @2
        get_local 3
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 2
        i32.const 150
        get_local 4
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        set_local 4
        br 1 (;@1;)
      end
      get_local 2
      get_local 4
      i32.const 10
      i32.add
      i32.const 31
      i32.and
      i32.shl
      set_local 4
    end
    get_local 4
    i32.const 0
    get_local 4
    i32.sub
    get_local 1
    i32.const -1
    i32.gt_s
    select)
  (func $__fixsfdi (type 19) (param f32) (result i64)
    (local i32 i32 i32 i32 i64)
    i64.const 0
    set_local 5
    block  ;; label = @1
      block  ;; label = @2
        get_local 0
        i32.reinterpret/f32
        tee_local 1
        i32.const 2139095040
        i32.and
        tee_local 4
        i32.const 1065353216
        i32.lt_u
        br_if 0 (;@2;)
        get_local 4
        i32.const 23
        i32.shr_u
        tee_local 4
        i32.const -127
        i32.add
        tee_local 3
        i32.const 62
        i32.le_u
        br_if 1 (;@1;)
        i64.const 9223372036854775807
        i64.const -9223372036854775808
        get_local 1
        i32.const -1
        i32.gt_s
        select
        set_local 5
      end
      get_local 5
      return
    end
    get_local 1
    i32.const 8388607
    i32.and
    i32.const 8388608
    i32.or
    set_local 2
    block  ;; label = @1
      block  ;; label = @2
        get_local 3
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 2
        i32.const 150
        get_local 4
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        i64.extend_u/i32
        set_local 5
        br 1 (;@1;)
      end
      get_local 2
      i64.extend_u/i32
      get_local 4
      i32.const 42
      i32.add
      i32.const 63
      i32.and
      i64.extend_u/i32
      i64.shl
      set_local 5
    end
    get_local 5
    i64.const 0
    get_local 5
    i64.sub
    get_local 1
    i32.const -1
    i32.gt_s
    select)
  (func $__fixsfti (type 20) (param i32 f32)
    (local i32 i32 i32 i32 i32 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 6
    i32.store offset=4
    i64.const 0
    set_local 7
    i64.const 0
    set_local 8
    block  ;; label = @1
      block  ;; label = @2
        get_local 1
        i32.reinterpret/f32
        tee_local 2
        i32.const 2139095040
        i32.and
        tee_local 5
        i32.const 1065353216
        i32.lt_u
        br_if 0 (;@2;)
        get_local 5
        i32.const 23
        i32.shr_u
        tee_local 5
        i32.const -127
        i32.add
        tee_local 4
        i32.const 126
        i32.le_u
        br_if 1 (;@1;)
        i64.const 9223372036854775807
        i64.const -9223372036854775808
        get_local 2
        i32.const -1
        i32.gt_s
        tee_local 2
        select
        set_local 8
        i64.const -1
        i64.const 0
        get_local 2
        select
        set_local 7
      end
      get_local 0
      get_local 7
      i64.store
      get_local 0
      i32.const 8
      i32.add
      get_local 8
      i64.store
      i32.const 0
      get_local 6
      i32.const 16
      i32.add
      i32.store offset=4
      return
    end
    get_local 2
    i32.const 8388607
    i32.and
    i32.const 8388608
    i32.or
    set_local 3
    block  ;; label = @1
      block  ;; label = @2
        get_local 4
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 3
        i32.const 150
        get_local 5
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        i64.extend_u/i32
        set_local 7
        i64.const 0
        set_local 8
        br 1 (;@1;)
      end
      get_local 6
      get_local 3
      i64.extend_u/i32
      i64.const 0
      get_local 5
      i32.const 106
      i32.add
      i32.const 127
      i32.and
      call $__ashlti3
      get_local 6
      i32.const 8
      i32.add
      i64.load
      set_local 8
      get_local 6
      i64.load
      set_local 7
    end
    get_local 0
    get_local 7
    i64.const 0
    get_local 7
    i64.sub
    get_local 2
    i32.const -1
    i32.gt_s
    tee_local 2
    select
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 8
    i64.const 0
    get_local 8
    i64.sub
    get_local 7
    i64.const 0
    i64.ne
    i64.extend_u/i32
    i64.sub
    get_local 2
    select
    i64.store
    i32.const 0
    get_local 6
    i32.const 16
    i32.add
    i32.store offset=4)
  (func $__fixdfsi (type 21) (param f64) (result i32)
    (local i32 i32 i64 i64)
    i32.const 0
    set_local 2
    block  ;; label = @1
      block  ;; label = @2
        get_local 0
        i64.reinterpret/f64
        tee_local 3
        i64.const 52
        i64.shr_u
        tee_local 4
        i32.wrap/i64
        i32.const 2047
        i32.and
        tee_local 1
        i32.const 1023
        i32.lt_u
        br_if 0 (;@2;)
        get_local 1
        i32.const -1023
        i32.add
        i32.const 30
        i32.le_u
        br_if 1 (;@1;)
        i32.const 2147483647
        i32.const -2147483648
        get_local 3
        i64.const -1
        i64.gt_s
        select
        set_local 2
      end
      get_local 2
      return
    end
    get_local 3
    i64.const 4503599627370495
    i64.and
    i64.const 4503599627370496
    i64.or
    i64.const 1075
    get_local 4
    i64.sub
    i64.const 63
    i64.and
    i64.shr_u
    i32.wrap/i64
    tee_local 2
    i32.const 0
    get_local 2
    i32.sub
    get_local 3
    i64.const -1
    i64.gt_s
    select)
  (func $__fixdfdi (type 22) (param f64) (result i64)
    (local i32 i64 i64 i64)
    i64.const 0
    set_local 4
    block  ;; label = @1
      block  ;; label = @2
        get_local 0
        i64.reinterpret/f64
        tee_local 2
        i64.const 52
        i64.shr_u
        tee_local 3
        i32.wrap/i64
        i32.const 2047
        i32.and
        tee_local 1
        i32.const 1023
        i32.lt_u
        br_if 0 (;@2;)
        get_local 1
        i32.const -1023
        i32.add
        tee_local 1
        i32.const 62
        i32.le_u
        br_if 1 (;@1;)
        get_local 2
        i64.const 63
        i64.shr_u
        i64.const 9223372036854775807
        i64.add
        set_local 4
      end
      get_local 4
      return
    end
    get_local 2
    i64.const 4503599627370495
    i64.and
    i64.const 4503599627370496
    i64.or
    set_local 4
    block  ;; label = @1
      block  ;; label = @2
        get_local 1
        i32.const 51
        i32.gt_u
        br_if 0 (;@2;)
        get_local 4
        i64.const 1075
        get_local 3
        i64.sub
        i64.const 63
        i64.and
        i64.shr_u
        set_local 4
        br 1 (;@1;)
      end
      get_local 4
      get_local 3
      i64.const 13
      i64.add
      i64.const 63
      i64.and
      i64.shl
      set_local 4
    end
    get_local 4
    i64.const 0
    get_local 4
    i64.sub
    get_local 2
    i64.const -1
    i64.gt_s
    select)
  (func $__fixdfti (type 23) (param i32 f64)
    (local i32 i32 i32 i64 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 4
    i32.store offset=4
    i64.const 0
    set_local 7
    i64.const 0
    set_local 8
    block  ;; label = @1
      block  ;; label = @2
        get_local 1
        i64.reinterpret/f64
        tee_local 5
        i64.const 52
        i64.shr_u
        tee_local 6
        i32.wrap/i64
        tee_local 2
        i32.const 2047
        i32.and
        tee_local 3
        i32.const 1023
        i32.lt_u
        br_if 0 (;@2;)
        get_local 3
        i32.const -1023
        i32.add
        tee_local 3
        i32.const 126
        i32.le_u
        br_if 1 (;@1;)
        i64.const 9223372036854775807
        i64.const -9223372036854775808
        get_local 5
        i64.const -1
        i64.gt_s
        tee_local 3
        select
        set_local 8
        i64.const -1
        i64.const 0
        get_local 3
        select
        set_local 7
      end
      get_local 0
      get_local 7
      i64.store
      get_local 0
      i32.const 8
      i32.add
      get_local 8
      i64.store
      i32.const 0
      get_local 4
      i32.const 16
      i32.add
      i32.store offset=4
      return
    end
    get_local 5
    i64.const 4503599627370495
    i64.and
    i64.const 4503599627370496
    i64.or
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        get_local 3
        i32.const 51
        i32.gt_u
        br_if 0 (;@2;)
        get_local 7
        i64.const 1075
        get_local 6
        i64.sub
        i64.const 63
        i64.and
        i64.shr_u
        set_local 7
        i64.const 0
        set_local 8
        br 1 (;@1;)
      end
      get_local 4
      get_local 7
      i64.const 0
      get_local 2
      i32.const 77
      i32.add
      i32.const 127
      i32.and
      call $__ashlti3
      get_local 4
      i32.const 8
      i32.add
      i64.load
      set_local 8
      get_local 4
      i64.load
      set_local 7
    end
    get_local 0
    get_local 7
    i64.const 0
    get_local 7
    i64.sub
    get_local 5
    i64.const -1
    i64.gt_s
    tee_local 3
    select
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 8
    i64.const 0
    get_local 8
    i64.sub
    get_local 7
    i64.const 0
    i64.ne
    i64.extend_u/i32
    i64.sub
    get_local 3
    select
    i64.store
    i32.const 0
    get_local 4
    i32.const 16
    i32.add
    i32.store offset=4)
  (func $__fixunssfsi (type 18) (param f32) (result i32)
    (local i32 i32 i32)
    i32.const 0
    set_local 3
    block  ;; label = @1
      get_local 0
      i32.reinterpret/f32
      tee_local 1
      i32.const 0
      i32.lt_s
      br_if 0 (;@1;)
      get_local 1
      i32.const 2139095040
      i32.and
      tee_local 2
      i32.const 1065353216
      i32.lt_u
      br_if 0 (;@1;)
      block  ;; label = @2
        get_local 2
        i32.const 23
        i32.shr_u
        tee_local 3
        i32.const -127
        i32.add
        tee_local 2
        i32.const 31
        i32.le_u
        br_if 0 (;@2;)
        get_local 1
        i32.const 31
        i32.shr_s
        i32.const -1
        i32.xor
        return
      end
      get_local 1
      i32.const 8388607
      i32.and
      i32.const 8388608
      i32.or
      set_local 1
      block  ;; label = @2
        get_local 2
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 1
        i32.const 150
        get_local 3
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        return
      end
      get_local 1
      get_local 3
      i32.const 10
      i32.add
      i32.const 31
      i32.and
      i32.shl
      set_local 3
    end
    get_local 3)
  (func $__fixunssfdi (type 19) (param f32) (result i64)
    (local i32 i32 i32 i64)
    i64.const 0
    set_local 4
    block  ;; label = @1
      get_local 0
      i32.reinterpret/f32
      tee_local 1
      i32.const 0
      i32.lt_s
      br_if 0 (;@1;)
      get_local 1
      i32.const 2139095040
      i32.and
      tee_local 3
      i32.const 1065353216
      i32.lt_u
      br_if 0 (;@1;)
      block  ;; label = @2
        get_local 3
        i32.const 23
        i32.shr_u
        tee_local 3
        i32.const -127
        i32.add
        tee_local 2
        i32.const 63
        i32.le_u
        br_if 0 (;@2;)
        get_local 1
        i32.const 31
        i32.shr_s
        i64.extend_s/i32
        i64.const -1
        i64.xor
        return
      end
      get_local 1
      i32.const 8388607
      i32.and
      i32.const 8388608
      i32.or
      set_local 1
      block  ;; label = @2
        get_local 2
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 1
        i32.const 150
        get_local 3
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        i64.extend_u/i32
        return
      end
      get_local 1
      i64.extend_u/i32
      get_local 3
      i32.const 42
      i32.add
      i32.const 63
      i32.and
      i64.extend_u/i32
      i64.shl
      set_local 4
    end
    get_local 4)
  (func $__fixunssfti (type 20) (param i32 f32)
    (local i32 i32 i32 i32 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 5
    i32.store offset=4
    i64.const 0
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 1
          i32.reinterpret/f32
          tee_local 2
          i32.const 0
          i32.lt_s
          br_if 0 (;@3;)
          get_local 2
          i32.const 2139095040
          i32.and
          tee_local 4
          i32.const 1065353216
          i32.lt_u
          br_if 0 (;@3;)
          get_local 4
          i32.const 23
          i32.shr_u
          tee_local 4
          i32.const -127
          i32.add
          tee_local 3
          i32.const 127
          i32.le_u
          br_if 1 (;@2;)
          get_local 2
          i32.const 31
          i32.shr_s
          i64.extend_s/i32
          i64.const -1
          i64.xor
          tee_local 6
          set_local 7
          br 2 (;@1;)
        end
        i64.const 0
        set_local 6
        br 1 (;@1;)
      end
      get_local 2
      i32.const 8388607
      i32.and
      i32.const 8388608
      i32.or
      set_local 2
      block  ;; label = @2
        get_local 3
        i32.const 22
        i32.gt_u
        br_if 0 (;@2;)
        get_local 2
        i32.const 150
        get_local 4
        i32.sub
        i32.const 31
        i32.and
        i32.shr_u
        i64.extend_u/i32
        set_local 6
        br 1 (;@1;)
      end
      get_local 5
      get_local 2
      i64.extend_u/i32
      i64.const 0
      get_local 4
      i32.const 106
      i32.add
      i32.const 127
      i32.and
      call $__ashlti3
      get_local 5
      i32.const 8
      i32.add
      i64.load
      set_local 7
      get_local 5
      i64.load
      set_local 6
    end
    get_local 0
    get_local 6
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 7
    i64.store
    i32.const 0
    get_local 5
    i32.const 16
    i32.add
    i32.store offset=4)
  (func $__fixunsdfsi (type 21) (param f64) (result i32)
    (local i32 i32 i64 i64)
    i32.const 0
    set_local 2
    block  ;; label = @1
      get_local 0
      i64.reinterpret/f64
      tee_local 3
      i64.const 0
      i64.lt_s
      br_if 0 (;@1;)
      get_local 3
      i64.const 52
      i64.shr_u
      tee_local 4
      i32.wrap/i64
      i32.const 2047
      i32.and
      tee_local 1
      i32.const 1023
      i32.lt_u
      br_if 0 (;@1;)
      block  ;; label = @2
        get_local 1
        i32.const -1023
        i32.add
        i32.const 31
        i32.le_u
        br_if 0 (;@2;)
        get_local 3
        i64.const 63
        i64.shr_s
        i32.wrap/i64
        i32.const -1
        i32.xor
        return
      end
      get_local 3
      i64.const 4503599627370495
      i64.and
      i64.const 4503599627370496
      i64.or
      i64.const 1075
      get_local 4
      i64.sub
      i64.const 63
      i64.and
      i64.shr_u
      i32.wrap/i64
      set_local 2
    end
    get_local 2)
  (func $__fixunsdfdi (type 22) (param f64) (result i64)
    (local i32 i64 i64 i64)
    i64.const 0
    set_local 4
    block  ;; label = @1
      get_local 0
      i64.reinterpret/f64
      tee_local 2
      i64.const 0
      i64.lt_s
      br_if 0 (;@1;)
      get_local 2
      i64.const 52
      i64.shr_u
      tee_local 3
      i32.wrap/i64
      i32.const 2047
      i32.and
      tee_local 1
      i32.const 1023
      i32.lt_u
      br_if 0 (;@1;)
      block  ;; label = @2
        get_local 1
        i32.const -1023
        i32.add
        tee_local 1
        i32.const 63
        i32.le_u
        br_if 0 (;@2;)
        get_local 2
        i64.const 63
        i64.shr_s
        i64.const -1
        i64.xor
        return
      end
      get_local 2
      i64.const 4503599627370495
      i64.and
      i64.const 4503599627370496
      i64.or
      set_local 2
      block  ;; label = @2
        get_local 1
        i32.const 51
        i32.gt_u
        br_if 0 (;@2;)
        get_local 2
        i64.const 1075
        get_local 3
        i64.sub
        i64.const 63
        i64.and
        i64.shr_u
        return
      end
      get_local 2
      get_local 3
      i64.const 13
      i64.add
      i64.const 63
      i64.and
      i64.shl
      set_local 4
    end
    get_local 4)
  (func $__fixunsdfti (type 23) (param i32 f64)
    (local i32 i32 i32 i64 i64 i64)
    i32.const 0
    i32.const 0
    i32.load offset=4
    i32.const 16
    i32.sub
    tee_local 4
    i32.store offset=4
    i64.const 0
    set_local 7
    block  ;; label = @1
      block  ;; label = @2
        block  ;; label = @3
          get_local 1
          i64.reinterpret/f64
          tee_local 5
          i64.const 0
          i64.lt_s
          br_if 0 (;@3;)
          get_local 5
          i64.const 52
          i64.shr_u
          tee_local 6
          i32.wrap/i64
          tee_local 2
          i32.const 2047
          i32.and
          tee_local 3
          i32.const 1023
          i32.lt_u
          br_if 0 (;@3;)
          get_local 3
          i32.const -1023
          i32.add
          tee_local 3
          i32.const 127
          i32.le_u
          br_if 1 (;@2;)
          get_local 5
          i64.const 63
          i64.shr_s
          i64.const -1
          i64.xor
          tee_local 5
          set_local 7
          br 2 (;@1;)
        end
        i64.const 0
        set_local 5
        br 1 (;@1;)
      end
      get_local 5
      i64.const 4503599627370495
      i64.and
      i64.const 4503599627370496
      i64.or
      set_local 5
      block  ;; label = @2
        get_local 3
        i32.const 51
        i32.gt_u
        br_if 0 (;@2;)
        get_local 5
        i64.const 1075
        get_local 6
        i64.sub
        i64.const 63
        i64.and
        i64.shr_u
        set_local 5
        br 1 (;@1;)
      end
      get_local 4
      get_local 5
      i64.const 0
      get_local 2
      i32.const 77
      i32.add
      i32.const 127
      i32.and
      call $__ashlti3
      get_local 4
      i32.const 8
      i32.add
      i64.load
      set_local 7
      get_local 4
      i64.load
      set_local 5
    end
    get_local 0
    get_local 5
    i64.store
    get_local 0
    i32.const 8
    i32.add
    get_local 7
    i64.store
    i32.const 0
    get_local 4
    i32.const 16
    i32.add
    i32.store offset=4)
  (table (;0;) 0 anyfunc)
  (memory (;0;) 17)
  (export "memory" (memory 0))
  (export "rust_eh_personality" (func $rust_eh_personality))
  (export "memcpy" (func $memcpy))
  (export "memmove" (func $memmove))
  (export "memset" (func $memset))
  (export "memcmp" (func $memcmp))
  (export "__subsf3" (func $__subsf3))
  (export "__subdf3" (func $__subdf3))
  (export "__udivsi3" (func $__udivsi3))
  (export "__umodsi3" (func $__umodsi3))
  (export "__udivmodsi4" (func $__udivmodsi4))
  (export "__udivdi3" (func $__udivdi3))
  (export "__udivmoddi4" (func $__udivmoddi4))
  (export "__umoddi3" (func $__umoddi3))
  (export "__udivti3" (func $__udivti3))
  (export "__udivmodti4" (func $__udivmodti4))
  (export "__umodti3" (func $__umodti3))
  (export "__addsf3" (func $__addsf3))
  (export "__adddf3" (func $__adddf3))
  (export "__muldi3" (func $__muldi3))
  (export "__multi3" (func $__multi3))
  (export "__mulosi4" (func $__mulosi4))
  (export "__mulodi4" (func $__mulodi4))
  (export "__muloti4" (func $__muloti4))
  (export "__powisf2" (func $__powisf2))
  (export "__powidf2" (func $__powidf2))
  (export "__mulsf3" (func $__mulsf3))
  (export "__muldf3" (func $__muldf3))
  (export "__divsi3" (func $__divsi3))
  (export "__divdi3" (func $__divdi3))
  (export "__divti3" (func $__divti3))
  (export "__modsi3" (func $__modsi3))
  (export "__moddi3" (func $__moddi3))
  (export "__modti3" (func $__modti3))
  (export "__divmodsi4" (func $__divmodsi4))
  (export "__divmoddi4" (func $__divmoddi4))
  (export "__divsf3" (func $__divsf3))
  (export "__divdf3" (func $__divdf3))
  (export "__ashldi3" (func $__ashldi3))
  (export "__ashlti3" (func $__ashlti3))
  (export "__ashrdi3" (func $__ashrdi3))
  (export "__ashrti3" (func $__ashrti3))
  (export "__lshrdi3" (func $__lshrdi3))
  (export "__lshrti3" (func $__lshrti3))
  (export "__floatsisf" (func $__floatsisf))
  (export "__floatsidf" (func $__floatsidf))
  (export "__floatdidf" (func $__floatdidf))
  (export "__floattisf" (func $__floattisf))
  (export "__floattidf" (func $__floattidf))
  (export "__floatunsisf" (func $__floatunsisf))
  (export "__floatunsidf" (func $__floatunsidf))
  (export "__floatundidf" (func $__floatundidf))
  (export "__floatuntisf" (func $__floatuntisf))
  (export "__floatuntidf" (func $__floatuntidf))
  (export "__fixsfsi" (func $__fixsfsi))
  (export "__fixsfdi" (func $__fixsfdi))
  (export "__fixsfti" (func $__fixsfti))
  (export "__fixdfsi" (func $__fixdfsi))
  (export "__fixdfdi" (func $__fixdfdi))
  (export "__fixdfti" (func $__fixdfti))
  (export "__fixunssfsi" (func $__fixunssfsi))
  (export "__fixunssfdi" (func $__fixunssfdi))
  (export "__fixunssfti" (func $__fixunssfti))
  (export "__fixunsdfsi" (func $__fixunsdfsi))
  (export "__fixunsdfdi" (func $__fixunsdfdi))
  (export "__fixunsdfti" (func $__fixunsdfti))
  (data (i32.const 4) "\10\00\10\00"))
