# Working through Building a Debugger

* https://github.com/tartanllama/sdb

## Notes

* `ptrace: Operation not permitted`
  * https://rajeeshknambiar.wordpress.com/2015/07/16/attaching-debugger-and-ptrace_scope/
    * `echo 0 > /proc/sys/kernel/yama/ptrace_scope` temporarily allows attaching to non-child proocesses
    * /etc/sysctl.d/10-ptrace.conf
      * If `kernel.yama.ptrace_scope` is modified to 0 then permanently allows attaching to non-child processes
