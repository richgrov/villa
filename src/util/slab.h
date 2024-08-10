#include "debug_assert.h"

#include <string.h>

#define SIMULO_INVALID_SLAB_KEY -1

static inline int slab_get_next_id(void *obj) {
   return *((int *)obj);
}

static inline void slab_set_next_id(void *obj, int id) {
   *((int *)obj) = id;
}

static inline void slab_init(void *objects, size_t num_objects, size_t obj_size) {
   for (int i = 0; i < num_objects; ++i) {
      void *obj_addr = ((char *)objects) + (obj_size * i);
      if (i == num_objects - 1) {
         slab_set_next_id(obj_addr, SIMULO_INVALID_SLAB_KEY);
      } else {
         slab_set_next_id(obj_addr, i + 1);
      }
   }
}

static inline void slab_reclaim(void *objects, size_t object_size, int obj_id, int *next_avail_id) {
   void *obj_addr = ((char *)objects) + (object_size * obj_id);
   slab_set_next_id(obj_addr, *next_avail_id);
   *next_avail_id = obj_id;
}

#undef SLAB_TYPE
#undef SLAB_LENGTH
#undef SLAB_NAME
