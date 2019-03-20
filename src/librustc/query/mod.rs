use crate::ty::query::QueryDescription;
use crate::ty::query::queries;
use crate::ty::TyCtxt;
use crate::ty;
use crate::hir::def_id::CrateNum;
use crate::dep_graph::SerializedDepNodeIndex;
use std::borrow::Cow;

// Each of these queries corresponds to a function pointer field in the
// `Providers` struct for requesting a value of that type, and a method
// on `tcx: TyCtxt` (and `tcx.at(span)`) for doing that request in a way
// which memoizes and does dep-graph tracking, wrapping around the actual
// `Providers` that the driver creates (using several `rustc_*` crates).
//
// The result type of each query must implement `Clone`, and additionally
// `ty::query::values::Value`, which produces an appropriate placeholder
// (error) value if the query resulted in a query cycle.
// Queries marked with `fatal_cycle` do not need the latter implementation,
// as they will raise an fatal error on query cycles instead.
rustc_queries! {
    Other {
        /// Records the type of every item.
        query type_of(key: DefId) -> Ty<'tcx> {
            cache { key.is_local() }
        }

        /// Maps from the `DefId` of an item (trait/struct/enum/fn) to its
        /// associated generics.
        query generics_of(key: DefId) -> &'tcx ty::Generics {
            cache { key.is_local() }
            load_cached(tcx, id) {
                let generics: Option<ty::Generics> = tcx.queries.on_disk_cache
                                                        .try_load_query_result(tcx, id);
                generics.map(|x| tcx.alloc_generics(x))
            }
        }

        /// Maps from the `DefId` of an item (trait/struct/enum/fn) to the
        /// predicates (where-clauses) that must be proven true in order
        /// to reference it. This is almost always the "predicates query"
        /// that you want.
        ///
        /// `predicates_of` builds on `predicates_defined_on` -- in fact,
        /// it is almost always the same as that query, except for the
        /// case of traits. For traits, `predicates_of` contains
        /// an additional `Self: Trait<...>` predicate that users don't
        /// actually write. This reflects the fact that to invoke the
        /// trait (e.g., via `Default::default`) you must supply types
        /// that actually implement the trait. (However, this extra
        /// predicate gets in the way of some checks, which are intended
        /// to operate over only the actual where-clauses written by the
        /// user.)
        query predicates_of(_: DefId) -> Lrc<ty::GenericPredicates<'tcx>> {}

        query native_libraries(_: CrateNum) -> Lrc<Vec<NativeLibrary>> {
            desc { "looking up the native libraries of a linked crate" }
        }
    }

    Codegen {
        query is_panic_runtime(_: CrateNum) -> bool {
            fatal_cycle
            desc { "checking if the crate is_panic_runtime" }
        }
    }

    Codegen {
        /// Set of all the `DefId`s in this crate that have MIR associated with
        /// them. This includes all the body owners, but also things like struct
        /// constructors.
        query mir_keys(_: CrateNum) -> Lrc<DefIdSet> {
            desc { "getting a list of all mir_keys" }
        }

        /// Maps DefId's that have an associated Mir to the result
        /// of the MIR qualify_consts pass. The actual meaning of
        /// the value isn't known except to the pass itself.
        query mir_const_qualif(key: DefId) -> (u8, Lrc<BitSet<mir::Local>>) {
            cache { key.is_local() }
        }

        /// Fetch the MIR for a given `DefId` right after it's built - this includes
        /// unreachable code.
        query mir_built(_: DefId) -> &'tcx Steal<mir::Mir<'tcx>> {}

        /// Fetch the MIR for a given `DefId` up till the point where it is
        /// ready for const evaluation.
        ///
        /// See the README for the `mir` module for details.
        query mir_const(_: DefId) -> &'tcx Steal<mir::Mir<'tcx>> {
            no_hash
        }

        query mir_validated(_: DefId) -> &'tcx Steal<mir::Mir<'tcx>> {
            no_hash
        }

        /// MIR after our optimization passes have run. This is MIR that is ready
        /// for codegen. This is also the only query that can fetch non-local MIR, at present.
        query optimized_mir(key: DefId) -> &'tcx mir::Mir<'tcx> {
            cache { key.is_local() }
            load_cached(tcx, id) {
                let mir: Option<crate::mir::Mir<'tcx>> = tcx.queries.on_disk_cache
                                                            .try_load_query_result(tcx, id);
                mir.map(|x| tcx.alloc_mir(x))
            }
        }
    }
}
