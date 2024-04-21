#ifndef SIMULO_UTIL_SLAB_H_
#define SIMULO_UTIL_SLAB_H_

#include "debug_assert.h"

namespace simulo {

static constexpr int kInvalidSlabKey = -1;

template <class T, int Length> class Slab {
public:
   explicit Slab() : next_available_(0) {
      static_assert(Length > 0);

      for (int i = 0; i < Length; ++i) {
         if (i == Length - 1) {
            objects_[i].next = kInvalidSlabKey;
         } else {
            objects_[i].next = i + 1;
         }
      }
   }

   [[nodiscard]] T &get(const int index) {
      return get_cell(index).value;
   }

   /**
    * Returns `kInvalidSlabKey` if allocation fails
    */
   int insert(const T value) {
      if (next_available_ == kInvalidSlabKey) {
         return kInvalidSlabKey;
      }

      int key = next_available_;
      auto &cell = get_cell(key);
      next_available_ = cell.next;
      cell.value = value;
      return key;
   }

   void release(const int key) {
      get_cell(key).next = next_available_;
      next_available_ = key;
   }

private:
   union Cell {
      int next;
      T value;

      Cell() : next(0) {}
   };

   Cell &get_cell(const int index) {
      SIMULO_DEBUG_ASSERT(index >= 0 && index < Length, "index {} out of slab range {}", index,
                          Length);
      return objects_[index];
   }

   Cell objects_[Length];
   int next_available_;
};

} // namespace simulo

#endif // !SIMULO_UTIL_SLAB_H_
