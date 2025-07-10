(module
  ;; Imports
  (import "env" "memory" (memory 1))
  (import "host" "recv_message" (func $host_recv_message (result i32)))
  (import "host" "send_message" (func $host_send_message (param i32) (result i32)))
  (import "host" "read_memory" (func $host_read_memory (param i32 i32 i32) (result i32)))
  (import "host" "write_memory" (func $host_write_memory (param i32 i32 i32) (result i32)))
  (import "host" "rpc_call" (func $host_rpc_call (param i32 i32 i32) (result i32)))
  
  ;; Exports
  (export "process_message" (func $process_message))
  (export "memory" (memory 0))
  
  ;; Data section for message storage
  (data (i32.const 0) "Hello from WASM!")
  
  ;; Process message function - echoes back the received message
  (func $process_message (result i32)
    ;; Receive message from host
    call $host_recv_message
    
    ;; Send it back
    call $host_send_message
  )
)
