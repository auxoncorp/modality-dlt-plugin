Common options:
- How to construct timeline name
- how to construct event name
- decoder ring options
  - fibex path
  - other?


Collector options:
- Listening
  - tcp dlt
    - port: default 3490
  - udl dlt
    - port: default 3490
  - serial dlt
    - device (ttyS0, etc)
    - baud rate (115200)
    - there's a 'serial header'?
  - serial ascii (???)
  - fifo to listen on?
    - seems to default to /tmp/dlt sometimes?
  
- push/pull
  - could be a server, or could connect to another server and receive data that way
  - presumably people already have a server of some kind? (dlt-daemon does the job)

Importer options:
- file to open ("dlt stream file")
  - opt "with serial header"



-------------
protocol
- send 'get log info'?
# Creating test data
- Run as root
- run as root:
```
   sudo dlt-daemon &
   dlt-receive -o foo.dlt
   sudo /usr/lib/libdlt-examples/dlt-example-user -n 5 -l 3 "fooo"
```

dlt-example-user connects via the named pipe (/tmp/dlt), not the tcp port.

--------------------

decoder ring
- for non-verbose messages
- fibex seems to be one option, maybe "autosar xml" is another (arxml)
