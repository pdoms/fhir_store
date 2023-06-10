# A database for fhir resources written in Rust

That's the plan.

## IMPORTANT: I am prototyping at the moment. This code is not usable at all.

I wanted it to be public nevertheless. 

### Roadmap Sketch

These are the next steps, defined as of June 10, 2023. Optimizations and refactoring wil be most likely handled along to the way, or after this. 

#### 1. Be able to create different [Patient](`http://hl7.org/fhir/patient.html`) Resources.
This entails, that deserialization from json will cover all necessary [Data Types](`http://hl7.org/fhir/datatypes.html`) These should be serialized into a custom binary format (which is part of the prototyping process) and persisted to disk. It then should be possible to retrieve them by using an identifier associated at creation. The returning format will be json, as one of the goals is, which yet have to be formally defined, to directly integrate the Fhir Store with the [FHIR REST](`http://hl7.org/fhir/http.html`) specification. 

#### 2. Use Index
Implement indexing on the database assigned identifier,use a btree for that. Just a simple implementation to get it running. It is too dependend on the pager to go too deep into any Implementation details. 

#### 3 Make it a DBMS
The following steps are collected under one step, as they will most likely be handled simultaneously. Or in other words, I haven't spend much defining any of these, I'm still researching.

##### 3.1 Handle Paging
Implemet a pager for i/o.

##### 3.2 Concurrency
Memory Management,access, reads and writes.


##### 3.3 Define ACID 
Will start with transactions.





