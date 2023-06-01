#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

constexpr static const uint8_t DEFAULT_TRACE = UINT8_MAX;

/// This mask extracts a few bits from address, and use it as index to the space map table.
/// This constant is specially picked for the current heap range (HEAP_STRAT/HEAP_END), and the space size (MAX_SPACE_EXTENT).
/// If any of these changes, the test `test_address_arithmetic()` may fail, and this constant will need to be updated.
/// Currently our spaces are using address range 0x0000_0200_0000_0000 to 0x0000_2200_0000_0000 (with a maximum of 16 spaces).
/// When masked with this constant, the index is 1 to 16. If we mask any arbitrary address with this mask, we will get 0 to 31 (32 entries).
constexpr static const uintptr_t SFTSpaceMap_ADDRESS_MASK = 69269232549888;

/// The max object size for immix: half of a block
constexpr static const uintptr_t MAX_IMMIX_OBJECT_SIZE = (Block_BYTES >> 1);

/// Mark/sweep memory for block-level only
constexpr static const bool BLOCK_ONLY = false;

/// Make every GC a defragment GC. (for debugging)
constexpr static const bool STRESS_DEFRAG = false;

/// Mark every allocated block as defragmentation source before GC. (for debugging)
/// Set both this and `STRESS_DEFRAG` to true to make Immix move as many objects as possible.
constexpr static const bool DEFRAG_EVERY_BLOCK = false;

/// In some cases/settings, Immix may never move objects.
/// Currently we only have two cases where we move objects: 1. defrag, 2. nursery copy.
/// If we do neither, we will not move objects.
/// If we have other reasons to move objects, we need to add them here.
constexpr static const bool NEVER_MOVE_OBJECTS = (~DEFRAG && ~PREFER_COPY_ON_NURSERY_GC);

/// Mark lines when scanning objects.
/// Otherwise, do it at mark time.
constexpr static const bool MARK_LINE_AT_SCAN_TIME = true;

/// For each MarkCompact object, we need one extra word for storing forwarding pointer (Lisp-2 implementation).
/// Note that considering the object alignment, we may end up allocating/reserving more than one word per object.
/// See [`MarkCompactSpace::HEADER_RESERVED_IN_BYTES`].
constexpr static const uintptr_t GC_EXTRA_HEADER_WORD = 1;

constexpr static const uintptr_t MAX_NON_LOS_ALLOC_BYTES_COPYING_PLAN = (16 << LOG_BYTES_IN_PAGE);

/// Full heap collection as nursery GC.
constexpr static const bool FULL_NURSERY_GC = false;

constexpr static const uintptr_t LOG_BYTES_IN_REGION = 22;

constexpr static const uintptr_t BYTES_IN_REGION = (1 << LOG_BYTES_IN_REGION);

constexpr static const uintptr_t REGION_MASK = (BYTES_IN_REGION - 1);

constexpr static const uintptr_t LOG_PAGES_IN_REGION = (LOG_BYTES_IN_REGION - (uintptr_t)LOG_BYTES_IN_PAGE);

constexpr static const uintptr_t PAGES_IN_REGION = (1 << LOG_PAGES_IN_REGION);

///  * Modes.
constexpr static const uintptr_t INSTANCE_FIELD = 0;

constexpr static const uintptr_t ARRAY_ELEMENT = 1;

constexpr static const uint8_t LOG_BYTES_IN_BYTE = 0;

constexpr static const uintptr_t BYTES_IN_BYTE = 1;

constexpr static const uint8_t LOG_BITS_IN_BYTE = 3;

constexpr static const uintptr_t BITS_IN_BYTE = (1 << LOG_BITS_IN_BYTE);

constexpr static const uint8_t LOG_BYTES_IN_MBYTE = 20;

constexpr static const uintptr_t BYTES_IN_MBYTE = (1 << LOG_BYTES_IN_MBYTE);

constexpr static const uint8_t LOG_BYTES_IN_KBYTE = 10;

constexpr static const uintptr_t BYTES_IN_KBYTE = (1 << LOG_BYTES_IN_KBYTE);

constexpr static const bool SUPPORT_CARD_SCANNING = false;

constexpr static const uintptr_t LOG_CARD_META_SIZE = 2;

constexpr static const uintptr_t LOG_CARD_UNITS = 10;

constexpr static const uintptr_t LOG_CARD_GRAIN = 0;

constexpr static const uintptr_t LOG_CARD_BYTES = (LOG_CARD_UNITS + LOG_CARD_GRAIN);

constexpr static const uintptr_t LOG_CARD_META_BYTES = ((LOG_BYTES_IN_REGION - LOG_CARD_BYTES) + LOG_CARD_META_SIZE);

constexpr static const uintptr_t LOG_CARD_META_PAGES = (LOG_CARD_META_BYTES - (uintptr_t)LOG_BYTES_IN_PAGE);

constexpr static const uintptr_t CARD_MASK = ((1 << LOG_CARD_BYTES) - 1);

///  * Lazy sweeping - controlled from here because PlanConstraints needs to  * tell the VM that we need to support linear scan.
constexpr static const bool LAZY_SWEEP = true;

constexpr static const uint8_t LOG_BYTES_IN_CHAR = 1;

constexpr static const uintptr_t BYTES_IN_CHAR = (1 << LOG_BYTES_IN_CHAR);

constexpr static const uint8_t LOG_BITS_IN_CHAR = (LOG_BITS_IN_BYTE + LOG_BYTES_IN_CHAR);

constexpr static const uintptr_t BITS_IN_CHAR = (1 << LOG_BITS_IN_CHAR);

constexpr static const uint8_t LOG_BYTES_IN_SHORT = 1;

constexpr static const uintptr_t BYTES_IN_SHORT = (1 << LOG_BYTES_IN_SHORT);

constexpr static const uint8_t LOG_BITS_IN_SHORT = (LOG_BITS_IN_BYTE + LOG_BYTES_IN_SHORT);

constexpr static const uintptr_t BITS_IN_SHORT = (1 << LOG_BITS_IN_SHORT);

constexpr static const uint8_t LOG_BYTES_IN_INT = 2;

constexpr static const uintptr_t BYTES_IN_INT = (1 << LOG_BYTES_IN_INT);

constexpr static const uint8_t LOG_BITS_IN_INT = (LOG_BITS_IN_BYTE + LOG_BYTES_IN_INT);

constexpr static const uintptr_t BITS_IN_INT = (1 << LOG_BITS_IN_INT);

constexpr static const uint8_t LOG_BYTES_IN_LONG = 3;

constexpr static const uintptr_t BYTES_IN_LONG = (1 << LOG_BYTES_IN_LONG);

constexpr static const uint8_t LOG_BITS_IN_LONG = (LOG_BITS_IN_BYTE + LOG_BYTES_IN_LONG);

constexpr static const uintptr_t BITS_IN_LONG = (1 << LOG_BITS_IN_LONG);

constexpr static const uint8_t LOG_BYTES_IN_ADDRESS = 2;

constexpr static const uint8_t LOG_BYTES_IN_ADDRESS = 3;

constexpr static const uintptr_t BYTES_IN_ADDRESS = (1 << LOG_BYTES_IN_ADDRESS);

constexpr static const uintptr_t LOG_BITS_IN_ADDRESS = ((uintptr_t)LOG_BITS_IN_BYTE + (uintptr_t)LOG_BYTES_IN_ADDRESS);

constexpr static const uintptr_t BITS_IN_ADDRESS = (1 << LOG_BITS_IN_ADDRESS);

constexpr static const uint8_t LOG_BYTES_IN_WORD = LOG_BYTES_IN_ADDRESS;

constexpr static const uintptr_t BYTES_IN_WORD = (1 << LOG_BYTES_IN_WORD);

constexpr static const uintptr_t LOG_BITS_IN_WORD = ((uintptr_t)LOG_BITS_IN_BYTE + (uintptr_t)LOG_BYTES_IN_WORD);

constexpr static const uintptr_t BITS_IN_WORD = (1 << LOG_BITS_IN_WORD);

constexpr static const uint8_t LOG_BYTES_IN_PAGE = 12;

constexpr static const uintptr_t BYTES_IN_PAGE = (1 << LOG_BYTES_IN_PAGE);

constexpr static const uintptr_t LOG_BITS_IN_PAGE = ((uintptr_t)LOG_BITS_IN_BYTE + (uintptr_t)LOG_BYTES_IN_PAGE);

constexpr static const uintptr_t BITS_IN_PAGE = (1 << LOG_BITS_IN_PAGE);

constexpr static const uint8_t LOG_BYTES_IN_ADDRESS_SPACE = (uint8_t)BITS_IN_ADDRESS;

constexpr static const uint8_t LOG_MIN_OBJECT_SIZE = LOG_BYTES_IN_WORD;

constexpr static const uintptr_t MIN_OBJECT_SIZE = (1 << LOG_MIN_OBJECT_SIZE);

/// The default nursery space size.
constexpr static const uintptr_t NURSERY_SIZE = ((1 << 20) << LOG_BYTES_IN_MBYTE);

/// The default nursery space size.
constexpr static const uintptr_t NURSERY_SIZE = (32 << LOG_BYTES_IN_MBYTE);

/// The default min nursery size. This does not affect the actual space we create as nursery. It is
/// only used in the GC trigger check.
constexpr static const uintptr_t DEFAULT_MIN_NURSERY = (2 << LOG_BYTES_IN_MBYTE);

/// The default min nursery size. This does not affect the actual space we create as nursery. It is
/// only used in the GC trigger check.
constexpr static const uintptr_t DEFAULT_MIN_NURSERY = (2 << LOG_BYTES_IN_MBYTE);

/// The default max nursery size. This does not affect the actual space we create as nursery. It is
/// only used in the GC trigger check.
constexpr static const uintptr_t DEFAULT_MAX_NURSERY = ((1 << 20) << LOG_BYTES_IN_MBYTE);

/// The default max nursery size. This does not affect the actual space we create as nursery. It is
/// only used in the GC trigger check.
constexpr static const uintptr_t DEFAULT_MAX_NURSERY = (32 << LOG_BYTES_IN_MBYTE);

///  * log_2 of the maximum number of spaces a Plan can support.
constexpr static const uintptr_t LOG_MAX_SPACES = 4;

///  * Maximum number of spaces a Plan can support.
constexpr static const uintptr_t MAX_SPACES = (1 << LOG_MAX_SPACES);

///  * In a 64-bit addressing model, each space is the same size, given  * by this constant.  At the moment, we require that the number of  * pages in a space fit into a 32-bit signed int, so the maximum  * size of this constant is 41 (assuming 4k pages).
constexpr static const uintptr_t LOG_SPACE_SIZE_64 = 41;

/// log_2 of the addressable virtual space.
constexpr static const uintptr_t LOG_ADDRESS_SPACE = 47;

constexpr static const uintptr_t LOG_ADDRESS_SPACE = 32;

///  * log_2 of the coarsest unit of address space allocation.  * <p>  * In the 32-bit VM layout, this determines the granularity of  * allocation in a discontigouous space.  In the 64-bit layout,  * this determines the growth factor of the large contiguous spaces  * that we provide.
constexpr static const uintptr_t LOG_BYTES_IN_CHUNK = 22;

/// Coarsest unit of address space allocation.
constexpr static const uintptr_t BYTES_IN_CHUNK = (1 << LOG_BYTES_IN_CHUNK);

constexpr static const uintptr_t CHUNK_MASK = ((1 << LOG_BYTES_IN_CHUNK) - 1);

/// Coarsest unit of address space allocation, in pages
constexpr static const uintptr_t PAGES_IN_CHUNK = (1 << (LOG_BYTES_IN_CHUNK - (uintptr_t)LOG_BYTES_IN_PAGE));

/// log_2 of the maximum number of chunks we need to track.  Only used in 32-bit layout.
constexpr static const uintptr_t LOG_MAX_CHUNKS = (LOG_ADDRESS_SPACE - LOG_BYTES_IN_CHUNK);

/// Maximum number of chunks we need to track.  Only used in 32-bit layout.
constexpr static const uintptr_t MAX_CHUNKS = (1 << LOG_MAX_CHUNKS);

///  * An upper bound on the extent of any space in the  * current memory layout
constexpr static const uintptr_t LOG_SPACE_EXTENT = LOG_SPACE_SIZE_64;

constexpr static const uintptr_t LOG_SPACE_EXTENT = 31;

///  * An upper bound on the extent of any space in the  * current memory layout
constexpr static const uintptr_t MAX_SPACE_EXTENT = (1 << LOG_SPACE_EXTENT);

/// Granularity at which we map and unmap virtual address space in the heap
constexpr static const uintptr_t LOG_MMAP_CHUNK_BYTES = LOG_BYTES_IN_CHUNK;

constexpr static const uintptr_t MMAP_CHUNK_BYTES = (1 << LOG_MMAP_CHUNK_BYTES);

/// log_2 of the number of pages in a 64-bit space
constexpr static const uintptr_t LOG_PAGES_IN_SPACE64 = (LOG_SPACE_SIZE_64 - (uintptr_t)LOG_BYTES_IN_PAGE);

/// The number of pages in a 64-bit space
constexpr static const uintptr_t PAGES_IN_SPACE64 = (1 << LOG_PAGES_IN_SPACE64);

///  * Number of bits to shift a space index into/out of a virtual address.
constexpr static const uintptr_t SPACE_SHIFT_64 = 0;

constexpr static const uintptr_t SPACE_SHIFT_64 = LOG_SPACE_SIZE_64;

///  * Bitwise mask to isolate a space index in a virtual address.  *  * We can't express this constant in a 32-bit environment, hence the  * conditional definition.
constexpr static const uintptr_t SPACE_MASK_64 = 0;

constexpr static const uintptr_t SPACE_MASK_64 = (((1 << LOG_MAX_SPACES) - 1) << SPACE_SHIFT_64);

constexpr static const uintptr_t SPACE_SIZE_64 = (1 << LOG_SPACE_SIZE_64);

constexpr static const uintptr_t SPACE_SIZE_64 = MAX_SPACE_EXTENT;

constexpr static const bool VERBOSE = true;

/// When we count page usage of library malloc, we assume they allocate in pages. For some malloc implementations,
/// they may use a larger page (e.g. mimalloc's 64K page). For libraries that we are not sure, we assume they use
/// normal 4k pages.
constexpr static const uintptr_t BYTES_IN_MALLOC_PAGE = (1 << LOG_BYTES_IN_MALLOC_PAGE);

constexpr static const uintptr_t LOG_MAX_GLOBAL_SIDE_METADATA_SIZE = (LOG_ADDRESS_SPACE - LOG_GLOBAL_SIDE_METADATA_WORST_CASE_RATIO);

constexpr static const uintptr_t MAX_PHASES = (1 << 12);

constexpr static const uintptr_t MAX_COUNTERS = 100;

constexpr static const int32_t FAILURE = -1;

constexpr static const int32_t MAX_HEADS = 128;

constexpr static const int32_t MAX_UNITS = ((((1 << UNIT_BITS) - 1) - MAX_HEADS) - 1);

/// BarrierSelector describes which barrier to use.
///
/// This is used as an *indicator* for each plan to enable the correct barrier.
/// For example, immix can use this selector to enable different barriers for analysis.
///
/// VM bindings may also use this to enable the correct fast-path, if the fast-path is implemented in the binding.
struct BarrierSelector;

/// An empty entry for SFT.
struct EmptySpaceSFT;

/// This struct defines plan-specific constraints.
/// Most of the constraints are constants. Each plan should declare a constant of this struct,
/// and use the constant wherever possible. However, for plan-neutral implementations,
/// these constraints are not constant.
struct PlanConstraints;

/// This is used for plans to indicate the number of allocators reserved for the plan.
/// This is used as a parameter for creating allocator/space mapping.
/// A plan is required to reserve the first few allocators. For example, if n_bump_pointer is 1,
/// it means the first bump pointer allocator will be reserved for the plan (and the plan should
/// initialize its mapping itself), and the spaces in common/base plan will use the following bump
/// pointer allocators.
struct ReservedAllocators;

struct SpaceDescriptor;

/// Address represents an arbitrary address. This is designed to represent
/// address and do address arithmetic mostly in a safe way, and to allow
/// mark some operations as unsafe. This type needs to be zero overhead
/// (memory wise and time wise). The idea is from the paper
/// High-level Low-level Programming (VEE09) and JikesRVM.
using Address = uintptr_t;
constexpr static const Address Address_ZERO = 0;

















///  * Lowest virtual address available for MMTk to manage.
constexpr static const Address AVAILABLE_START = HEAP_START;

///  * Highest virtual address available for MMTk to manage.
constexpr static const Address AVAILABLE_END = HEAP_END;



extern "C" {

extern uintptr_t malloc_size(const void *ptr);

} // extern "C"
