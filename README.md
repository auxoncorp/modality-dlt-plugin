# modality-dlt-plugin

Modality reflector plugins for AUTOSAR DLT (Diagnostic Log and Trace).

## Configuration
### Common
These options are used by both the collector and the importer.
| Config Key                     | Environment Variable                        | Meaning                                                                                                                                                                                                   |
|:-------------------------------|:--------------------------------------------|:----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `timeline_from_ecu_id`         | `MODALITY_DLT_TIMELINE_FROM_ECU_ID`         | Should the ecu id be used as part of timeline identity and naming? Defaults to true.                                                                                                                      |
| `timeline_from_session_id`     | `MODALITY_DLT_TIMELINE_FROM_SESSION_ID`     | Should the session be used as part of timeline identity (not naming)? Defaults to true.                                                                                                                   |
| `timeline_from_application_id` | `MODALITY_DLT_TIMELINE_FROM_APPLICATION_ID` | Should the application id field be used as part of timeline identity and naming? Defaults to false.                                                                                                       |
| `timeline_from_context_id`     | `MODALITY_DLT_TIMELINE_FROM_CONTEXT_ID`     | Should the context id field be used as part of timeline identity and naming? Defaults to false.                                                                                                           |
|                                | `MODALITY_RUN_ID`                           | The run id to value to use in timeline metadata (`timeline.run_id`). This is used as the basis for the segmentation method used in the default Modality workspace. Defaults to a randomly generated uuid. |
|                                | `MODALITY_AUTH_TOKEN`                       | The content of the auth token to use when connecting to Modality. If this is not set, the auth token used by the Modality CLI is read from `~/.config/modality_cli/.user_auth_token`                      |
|                                | `MODALITY_HOST`                             | The hostname where the modality server is running.                                                                                                                                                        |

### Collector
These options are used by both the collector and the importer.
| Config Key | Environment Variable | Meaning                                                                       |
|:-----------|:---------------------|:------------------------------------------------------------------------------|
| `host`     | `MODALITY_DLT_HOST`  | The DLT host to connect to (with TCP). If not given, defaults to "localhost". |
| `port`     | `MODALITY_DLT_PORT`  | The TCP port to connect to on the DLT host. If not given, defaults to 3490.   |

### Importer
The importer currently has no specific options. The file to import is given on the command line.

## Adapter Concept Mapping
The following describes the default mapping between DLT concepts and Modality's concepts.

* Timeline creation is customizable, based on the `timeline_from_*`
  configuration options. By default, the ECU ID and Session Id are
  used to uniquely identify the timeline.
  * If multiple fields are used for timeline naming, they are
    separated with a `.` character.
  * All fields that are configured for timeline identification are
    made available as timeline metadata. (`timeline.ecu_id`,
    `timeline.session_id`, `timeline.application_id`,
    `timeline.context_id`)
  * If all of fields configured for timeline naming are missing or
    empty, the timeline is named "unnamed".

* Events are named based on the type of the message:
  * For messages with an extended header, this can be `log`, `application_trace`, `network_trace`, `control`
  * For messages without an extended header, this can be `verbose`, `non_verbose`, or `control`

* Event timestamps are assigned directly from the `timestamp` field of the standard header.

* Message type details
  * For log messages, the log level is stored as lowercase.
  * For application trace messages, the application trace type is stored as lower case.
  * For network trace messages, the network trace type is stored as lower case.
  * For control messages, the control type is stored as lower case.

* Verbose messages
  * The `event.payload_type` attribute is set to `verbose`
  * If there is a single unnamed payload value, it is converted and
    stored in the `event.payload` attribute.
  * If there there are multiple, payload attributes, they are stored
    as `event.payload.<name>`. For unnamed attributes, their ordinal
    position in the payload array is used instead of a name (starting
    from 0).
  * Payload values are converted directly to modality value types.
    * Boolean, float, and string values are converted directly, with no loss.
    * All integer values, both signed and unsigned, are converted to
      Modality integers. Some u128 values may be too large for
      Modality (the largest value supported value representation is
      i128); those values are dropped, and a warning is logged.
    * `raw` values are currently ignored.

* Non-verbose
  * The `event.payload_type` attribute is set to `non_verbose`
  * The `event.message_id` attribute is set to the event's message id.
  * The payload value is ignored.
  * In the future, this plugin will support decoding message IDs and
    payloads using an external data source.

* When importing from a file, storage header content is currently ignored.
