# micro_sp


## dummy 
A dummy driver (inverse of a micro_sp node) can be launched providing the argument bla bla bla

<img src='https://g.gravizo.com/svg?
 digraph G {
   micro_sp [shape=box];
   dummy [shape=box];
   micro_sp -> ref_1;
   micro_sp -> ref_2;
   ref_1 -> dummy;
   ref_2 -> dummy;
   dummy -> act_1 -> micro_sp;
   dummy -> act_2 -> micro_sp;
   dummy -> act_ref_1 -> micro_sp;
   dummy -> act_ref_2 -> micro_sp;
 }
'/>

## TODO
### high prioriry 
1.  [x] error handling, probably not done
2.  [ ] don't complicate, have parameterized
3.  [ ] layered (comp(param(inc)))
4.  [x] publish complete state
5.  [x] update command vars to match measured values on refresh (raar)
6.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
7.  [ ] where is the state? who owns the state?
8.  [x] proper command line arguments
9.  [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
10. [ ] the "start/stop running" should actually set the reference values     ?       
11. [x] have to add delay for dummy act             
12. [ ] add boolean variables
13. [x] add handshake kind
14. [ ] the algorithms should probably be solver agnostic
15. [x] move setup to parser
16. [ ] other command paradigm or are we done?
17. [ ] testing
18. [x] structure and modules
19. [ ] generate dummy driver from the model
20. [x] documentation, probably not done
21. [ ] dummy_value should be deeply integrated
22. [ ] proper readme
23. [x] proper get planning result
24. [ ] include readmes to benches to describe added constraints
25. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
26. [ ] benchmarks barman
27. [ ] benchmarks childsnack
28. [ ] benchmarks gripper
29. [ ] benchmarks hiking
30. [ ] study conversion from a pddl model to a real world "runneble" model
31. [ ] enable other solvers beside z3
32. [ ] make a more general parser for boilerplate domain ? maybe not
33. [ ] clean the warnings
34. [ ] improve runner and don't replan after every state change
35. [ ] gui needed asap
36. [ ] show current plan in gui
37. [x] look into parsing pddls: conclusion: won't work
38. [ ] fix command lifetime
39. [x] match published data to sink when fresh timeouts
40. [ ] compositional algorithm
41. [ ] when to call the planner
42. [ ] handle estimated
43. [x] move model to models
44. [x] check if measured value is in domain
45. [ ] flow graph to readme
46. [ ] maybe try to write a cdcl solver?
47. [ ] some theories? hmm...

### low priority
1. [ ] improve Qol
2. [ ] solver agnostic, yes, but...
3. [ ] could micro_sp also be planner agnostic? try with dfs/bfs
4. [ ] notion of costs, add optimization support?
5. [ ] nondeterminism
6. [ ] multiple goals
7. [ ] dummy in var domain default or eliminate
8. [ ] dockerize
9. [ ] maybe some quality of life for modeling
10. [ ] generate both raar and invar paradigms from a hl model?
11. [ ] explore all paths and generate graph
    
### docs and tests

| where  | what          |               docs |              tests |
| :----- | :------------ | -----------------: | -----------------: |
| core   | compositional | :white_check_mark: | :white_check_mark: |
| core   | incremental   | :white_check_mark: | :white_check_mark: |
| core   | items         | :heavy_check_mark: | :heavy_check_mark: |
| core   | parameterized | :white_check_mark: | :white_check_mark: |
| core   | predicates    | :heavy_check_mark: | :heavy_check_mark: |
| models | blocksworld   | :white_check_mark: | :white_check_mark: |
| models | dummy_robot   | :white_check_mark: | :white_check_mark: |
| runner | dummy         | :white_check_mark: | :white_check_mark: |
| runner | node          | :white_check_mark: | :white_check_mark: |
| runner | publisher     | :white_check_mark: | :white_check_mark: |
| runner | receiver      | :white_check_mark: | :white_check_mark: |
| runner | sender        | :white_check_mark: | :white_check_mark: |
| runner | state         | :white_check_mark: | :white_check_mark: |
| runner | ticker        | :white_check_mark: | :white_check_mark: |
| utils  | core          | :white_check_mark: | :white_check_mark: |
| utils  | general       | :white_check_mark: | :white_check_mark: |
| utils  | runner        | :white_check_mark: | :white_check_mark: |
| main   | main          | :white_check_mark: | :white_check_mark: |