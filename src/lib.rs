#![doc = include_str!("../README.md")]
#![no_std]

/// Provides access to `super` blocks, a hypothetical language feature which
/// reorders inline `super { ... }` blocks into init statements at the top of the
/// inner scope.
///
/// # Examples
///
/// ```rust
/// # use std::sync::Arc;
/// # use soupa::soupa;
/// let foo = Arc::new(123usize);
///
/// let func = soupa!(move || {
///     // Any super { ... } expressions are eagerly evaluated and stored in
///     // temporary variables.
///     println!("Foo: {:?}", super { foo.clone() })
/// });
///
/// // The clone of foo was eagerly evaluated, so this foo can be dropped
/// // while func is live.
/// let _ = foo;
///
/// func();
/// ```
#[macro_export]
macro_rules! soupa {
    (
        @temps { $($temp:ident)* },
        @stack: {},
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack is empty
        // Output the initialization and body statements
        {
            $($init)*
            $($body)*
        }
    };

    (
        @temps { $next_ident:ident $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    super { $($next:tt)* }
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Process a super block into an init statement
        // Place an identifier of the declaration into the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        $next_ident
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: {
                $($init)*
                let $next_ident = { $($next)* };
            },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    { $($next:tt)* }
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a {} tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: {},
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    ( $($next:tt)* )
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a () tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: (),
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    [ $($next:tt)* ]
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a [] tree and place it onto the top of the stack
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: [],
                    @body: {},
                    @rest: { $($next)* },
                }
                {
                    @paren: $top_paren,
                    @body: { $($top_body)* },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: {
                    $next:tt
                    $($top_rest:tt)*
                },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Peel off a misc token and place in the top scope output
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        $next
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };

    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: {},
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        { $($next)* }
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: (),
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        ( $($next)* )
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: [],
                @body: { $($next:tt)* },
                @rest: { },
            }
            {
                @paren: $top_paren:tt,
                @body: { $($top_body:tt)* },
                @rest: { $($top_rest:tt)* },
            }
            $($stack:tt)*
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Top item on the stack is done
        // Combine it with the next item down
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {
                {
                    @paren: $top_paren,
                    @body: {
                        $($top_body)*
                        [ $($next)* ]
                    },
                    @rest: { $($top_rest)* },
                }
                $($stack)*
            },
            @init: { $($init)* },
            @body: { $($body)* },
        }
    };

    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: {},
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a {} wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                { $($next)* }
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: (),
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a () wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                ( $($next)* )
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: [],
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only a [] wrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                [ $($next)* ]
            },
        }
    };
    (
        @temps { $($temp:ident)* },
        @stack: {
            {
                @paren: None,
                @body: { $($next:tt)* },
                @rest: { },
            }
        },
        @init: { $($init:tt)* },
        @body: { $($body:tt)* },
    ) => {
        // Stack fully processed
        // Only an unwrapped body is left, so output it
        $crate::soupa! {
            @temps { $($temp)* },
            @stack: {},
            @init: { $($init)* },
            @body: {
                $($body)*
                $($next)*
            },
        }
    };

    (
        $($rest:tt)*
    ) => {
        // No other rule matches
        // Implies this is user supplied, so initialize with some temp variable names
        $crate::soupa! {
            @temps {
                __soupa_temp_a __soupa_temp_b __soupa_temp_c __soupa_temp_d __soupa_temp_e __soupa_temp_f __soupa_temp_g __soupa_temp_h __soupa_temp_i __soupa_temp_j __soupa_temp_k __soupa_temp_l __soupa_temp_m __soupa_temp_n __soupa_temp_o __soupa_temp_p __soupa_temp_q __soupa_temp_r __soupa_temp_s __soupa_temp_t __soupa_temp_u __soupa_temp_v __soupa_temp_w __soupa_temp_x __soupa_temp_y __soupa_temp_z
                __soupa_temp_aa __soupa_temp_ab __soupa_temp_ac __soupa_temp_ad __soupa_temp_ae __soupa_temp_af __soupa_temp_ag __soupa_temp_ah __soupa_temp_ai __soupa_temp_aj __soupa_temp_ak __soupa_temp_al __soupa_temp_am __soupa_temp_an __soupa_temp_ao __soupa_temp_ap __soupa_temp_aq __soupa_temp_ar __soupa_temp_as __soupa_temp_at __soupa_temp_au __soupa_temp_av __soupa_temp_aw __soupa_temp_ax __soupa_temp_ay __soupa_temp_az
                __soupa_temp_ba __soupa_temp_bb __soupa_temp_bc __soupa_temp_bd __soupa_temp_be __soupa_temp_bf __soupa_temp_bg __soupa_temp_bh __soupa_temp_bi __soupa_temp_bj __soupa_temp_bk __soupa_temp_bl __soupa_temp_bm __soupa_temp_bn __soupa_temp_bo __soupa_temp_bp __soupa_temp_bq __soupa_temp_br __soupa_temp_bs __soupa_temp_bt __soupa_temp_bu __soupa_temp_bv __soupa_temp_bw __soupa_temp_bx __soupa_temp_by __soupa_temp_bz
                __soupa_temp_ca __soupa_temp_cb __soupa_temp_cc __soupa_temp_cd __soupa_temp_ce __soupa_temp_cf __soupa_temp_cg __soupa_temp_ch __soupa_temp_ci __soupa_temp_cj __soupa_temp_ck __soupa_temp_cl __soupa_temp_cm __soupa_temp_cn __soupa_temp_co __soupa_temp_cp __soupa_temp_cq __soupa_temp_cr __soupa_temp_cs __soupa_temp_ct __soupa_temp_cu __soupa_temp_cv __soupa_temp_cw __soupa_temp_cx __soupa_temp_cy __soupa_temp_cz
                __soupa_temp_da __soupa_temp_db __soupa_temp_dc __soupa_temp_dd __soupa_temp_de __soupa_temp_df __soupa_temp_dg __soupa_temp_dh __soupa_temp_di __soupa_temp_dj __soupa_temp_dk __soupa_temp_dl __soupa_temp_dm __soupa_temp_dn __soupa_temp_do __soupa_temp_dp __soupa_temp_dq __soupa_temp_dr __soupa_temp_ds __soupa_temp_dt __soupa_temp_du __soupa_temp_dv __soupa_temp_dw __soupa_temp_dx __soupa_temp_dy __soupa_temp_dz
                __soupa_temp_ea __soupa_temp_eb __soupa_temp_ec __soupa_temp_ed __soupa_temp_ee __soupa_temp_ef __soupa_temp_eg __soupa_temp_eh __soupa_temp_ei __soupa_temp_ej __soupa_temp_ek __soupa_temp_el __soupa_temp_em __soupa_temp_en __soupa_temp_eo __soupa_temp_ep __soupa_temp_eq __soupa_temp_er __soupa_temp_es __soupa_temp_et __soupa_temp_eu __soupa_temp_ev __soupa_temp_ew __soupa_temp_ex __soupa_temp_ey __soupa_temp_ez
                __soupa_temp_fa __soupa_temp_fb __soupa_temp_fc __soupa_temp_fd __soupa_temp_fe __soupa_temp_ff __soupa_temp_fg __soupa_temp_fh __soupa_temp_fi __soupa_temp_fj __soupa_temp_fk __soupa_temp_fl __soupa_temp_fm __soupa_temp_fn __soupa_temp_fo __soupa_temp_fp __soupa_temp_fq __soupa_temp_fr __soupa_temp_fs __soupa_temp_ft __soupa_temp_fu __soupa_temp_fv __soupa_temp_fw __soupa_temp_fx __soupa_temp_fy __soupa_temp_fz
                __soupa_temp_ga __soupa_temp_gb __soupa_temp_gc __soupa_temp_gd __soupa_temp_ge __soupa_temp_gf __soupa_temp_gg __soupa_temp_gh __soupa_temp_gi __soupa_temp_gj __soupa_temp_gk __soupa_temp_gl __soupa_temp_gm __soupa_temp_gn __soupa_temp_go __soupa_temp_gp __soupa_temp_gq __soupa_temp_gr __soupa_temp_gs __soupa_temp_gt __soupa_temp_gu __soupa_temp_gv __soupa_temp_gw __soupa_temp_gx __soupa_temp_gy __soupa_temp_gz
                __soupa_temp_ha __soupa_temp_hb __soupa_temp_hc __soupa_temp_hd __soupa_temp_he __soupa_temp_hf __soupa_temp_hg __soupa_temp_hh __soupa_temp_hi __soupa_temp_hj __soupa_temp_hk __soupa_temp_hl __soupa_temp_hm __soupa_temp_hn __soupa_temp_ho __soupa_temp_hp __soupa_temp_hq __soupa_temp_hr __soupa_temp_hs __soupa_temp_ht __soupa_temp_hu __soupa_temp_hv __soupa_temp_hw __soupa_temp_hx __soupa_temp_hy __soupa_temp_hz
                __soupa_temp_ia __soupa_temp_ib __soupa_temp_ic __soupa_temp_id __soupa_temp_ie __soupa_temp_if __soupa_temp_ig __soupa_temp_ih __soupa_temp_ii __soupa_temp_ij __soupa_temp_ik __soupa_temp_il __soupa_temp_im __soupa_temp_in __soupa_temp_io __soupa_temp_ip __soupa_temp_iq __soupa_temp_ir __soupa_temp_is __soupa_temp_it __soupa_temp_iu __soupa_temp_iv __soupa_temp_iw __soupa_temp_ix __soupa_temp_iy __soupa_temp_iz
            },
            @stack: {
                {
                    @paren: None,
                    @body: {},
                    @rest: { $($rest)* },
                }
            },
            @init: {},
            @body: {},
        }
    };
}
