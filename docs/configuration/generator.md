# cdl-config-generator

CDL repository contains a tool that lets you generate configuration to your needs via interactive command prompt.
To run:
* checkout cdl repo locally
* install rustup
* run `cargo xtask config-generator -i` from cdl directory
> `-i` indicates interactive prompt; if omitted generator will expect command line arguments

You should see interactive prompt:
```bash
Finished dev [unoptimized + debuginfo] target(s) in 14.10s
 Running `.../CommonDataLayer/target/debug/xtask config-generator`
? Choose action ›
❯ review
  update
  save & quit
  abort
```

You can traverse options using arrow keys and accept options via enter key.

`review` lets you print each config individually to cmd in order to review it.  
`update` lets you change deployment configuration, eg. communication method (kafka, rabbitmq, grpc).  
`save & quit` lets you save deployment configuration to target directory; it also generates start.sh and stop.sh, for running and stopping cdl stack.  
`abort` quits immediately with no action on user system.  
