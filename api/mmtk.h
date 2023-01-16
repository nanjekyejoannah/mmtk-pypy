#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct PyPy;

extern "C" {

extern const uintptr_t FREE_LIST_ALLOCATOR_SIZE;

extern const uintptr_t GLOBAL_ALLOC_BIT_ADDRESS;

extern const uintptr_t GLOBAL_SIDE_METADATA_BASE_ADDRESS;

extern const uintptr_t GLOBAL_SIDE_METADATA_VM_BASE_ADDRESS;

extern const uintptr_t MMTK_MARK_COMPACT_HEADER_RESERVED_IN_BYTES;

void add_finalizer(ObjectReference object);

void add_phantom_candidate(ObjectReference reff);

void add_soft_candidate(ObjectReference reff);

void add_weak_candidate(ObjectReference reff);

void *bind_mutator();

void destroy_mutator(void *mutatorptr);

bool executable();

void flush_mutator(void *mutatorptr);

uintptr_t free_bytes();

AllocatorSelector get_allocator_mapping(AllocationSemantics allocator);

ObjectReference get_finalized_object();

uintptr_t get_max_non_los_default_alloc_bytes();

const char *get_mmtk_version();

void handle_user_collection_request(VMMutatorThread tls);

void harness_begin(uintptr_t _id);

void harness_end(uintptr_t _id);

void initialize_collection(VMThread tls);

bool is_in_mmtk_spaces(ObjectReference object);

bool is_mapped_address(Address addr);

Address last_heap_address();

const char *mmtk_active_barrier();

void mmtk_add_nmethod_oop(Address addr);

Address mmtk_alloc(void *mutator, uintptr_t size, uintptr_t align, intptr_t offset);

void mmtk_array_copy_post(Mutator<PyPy> *mutator, Address src, Address dst, uintptr_t count);

void mmtk_array_copy_pre(Mutator<PyPy> *mutator, Address src, Address dst, uintptr_t count);

void mmtk_harness_begin_impl();

void mmtk_harness_end_impl();

void mmtk_object_reference_write_post(Mutator<PyPy> *mutator,
                                      ObjectReference src,
                                      Address slot,
                                      ObjectReference target);

void mmtk_object_reference_write_pre(Mutator<PyPy> *mutator,
                                     ObjectReference src,
                                     Address slot,
                                     ObjectReference target);

void mmtk_object_reference_write_slow(Mutator<PyPy> *mutator,
                                      ObjectReference src,
                                      Address slot,
                                      ObjectReference target);

void mmtk_register_nmethod(Address nm);

bool mmtk_set_heap_size(uintptr_t min, uintptr_t max);

void mmtk_unregister_nmethod(Address nm);

void modify_check(ObjectReference object);

void post_alloc(Mutator<PyPy> *mutator,
                ObjectReference refer,
                uintptr_t bytes,
                AllocationSemantics allocator);

bool process(const char *name, const char *value);

bool process_bulk(const char *options);

void pypy_gc_init();

bool pypy_is_gc_initialized();

uintptr_t pypy_max_capacity();

void release_buffer(Address *ptr, uintptr_t length, uintptr_t capacity);

void scan_region();

void start_control_collector(VMWorkerThread tls, GCController<PyPy> *gc_controller);

void start_worker(VMWorkerThread tls, GCWorker<PyPy> *worker);

Address starting_heap_address();

uintptr_t total_bytes();

uintptr_t used_bytes();

bool will_never_move(ObjectReference object);

} // extern "C"
