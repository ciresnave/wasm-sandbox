(module
  ;; Imports
  (import "env" "memory" (memory 1))
  
  ;; Exports
  (export "add" (func $add))
  (export "memory" (memory 0))
  
  ;; Add function
  (func $add (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
)
