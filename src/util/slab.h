#ifndef SIMULO_UTIL_SLAB_H_
#define SIMULO_UTIL_SLAB_H_

#include "debug_assert.h"

#include <algorithm>
#include <utility>

namespace simulo {

static constexpr int kInvalidSlabKey = -1;

template <class T, int Length> class Slab {
public:
   explicit Slab() : next_available_(0) {
      static_assert(Length > 0);

      for (int i = 0; i < Length; ++i) {
         if (i == Length - 1) {
            get_storage(i).store_next(kInvalidSlabKey);
         } else {
            get_storage(i).store_next(i + 1);
         }
      }
   }

   ~Slab() {
      bool in_use[Length];
      std::fill_n(in_use, Length, true);

      int next_available = next_available_;
      while (next_available != kInvalidSlabKey) {
         in_use[next_available] = false;
         next_available = get_storage(next_available).next();
      }

      for (int i = 0; i < Length; ++i) {
         if (in_use[i]) {
            release(i);
         }
      }
   }

   [[nodiscard]] T &get(const int index) {
      return get_storage(index).value();
   }

   /**
    * Returns `kInvalidSlabKey` if allocation fails
    */
   template <class... Args> int emplace(Args &&...args) {
      if (next_available_ == kInvalidSlabKey) {
         return kInvalidSlabKey;
      }

      int key = next_available_;
      auto &storage = get_storage(key);
      next_available_ = storage.next();
      storage.store_value(std::forward<Args>(args)...);
      return key;
   }

   void release(const int key) {
      auto &storage = get_storage(key);
      storage.store_next(next_available_);
      next_available_ = key;
   }

private:
   struct Storage {
      static constexpr std::size_t kSize = std::max<std::size_t>({sizeof(int), sizeof(T)});
      static constexpr std::size_t kAlign = std::max<std::size_t>({alignof(int), alignof(T)});
      alignas(kAlign) unsigned char storage[kSize];

      int next() {
         auto ptr = reinterpret_cast<int *>(&storage);
         return *ptr;
      }

      void store_next(const int next) {
         auto ptr = reinterpret_cast<int *>(&storage);
         *ptr = next;
      }

      template <class... Args> void store_value(Args &&...args) {
         new (&storage) T(std::forward<Args>(args)...);
      }

      T &value() {
         auto ptr = reinterpret_cast<T *>(&storage);
         return *ptr;
      }
   };

   Storage &get_storage(const int index) {
      SIMULO_DEBUG_ASSERT(
          index >= 0 && index < Length, "index {} out of slab range {}", index, Length
      );
      return objects_[index];
   }

   Storage objects_[Length];
   int next_available_;
};

} // namespace simulo

#endif // !SIMULO_UTIL_SLAB_H_
