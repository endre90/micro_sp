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
2.  [x] don't complicate, have parameterized
3.  [x] layered (comp(param(inc)))
4.  [x] publish complete state
5.  [ ] use 'model' and 'instance'
6.  [ ] ok, going towards making the pddl parser :(
7.  [ ] think about how to parameterize modelling like the pddl people do
8.  [x] update command vars to match measured values on refresh (raar)
9.  [x] raar: act, ref and act_ref have to have the same domain and same r#type
10. [ ] where is the state? who owns the state?
11. [x] proper command line arguments
12. [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
13. [ ] the "start/stop running" should actually set the reference values     ?       
14. [x] have to add delay for dummy act             
15. [ ] add boolean variables
16. [x] add handshake kind
17. [ ] the algorithms should probably be solver agnostic
18. [x] move setup to parser
19. [ ] other command paradigm or are we done?
20. [ ] testing
21. [x] structure and modules
22. [ ] generate dummy driver from the model
23. [x] documentation, probably not done
24. [ ] dummy_value should be deeply integrated
25. [ ] proper readme
26. [x] proper get planning result
27. [ ] include readmes to benches to describe added constraints
28. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean
    2. [ ] micro_sp inc pure boolean
    3. [ ] micro_sp inc pure enumeration
    4. [ ] micro_sp seq enumerated boolean
    5. [ ] micro_sp seq pure boolean
    6. [ ] micro_sp seq pure enumeration
    7. [ ] incplan
29. [ ] benchmarks barman
30. [ ] benchmarks childsnack
31. [ ] benchmarks gripper
32. [ ] benchmarks hiking
33. [ ] study conversion from a pddl model to a real world "runneble" model
34. [ ] enable other solvers beside z3
35. [ ] make a more general parser for boilerplate domain ? maybe not
36. [ ] clean the warnings
37. [ ] improve runner and don't replan after every state change
38. [ ] gui needed asap
39. [ ] show current plan in gui
40. [x] look into parsing pddls: conclusion: won't work
41. [ ] fix command lifetime
42. [x] match published data to sink when fresh timeouts
43. [ ] compositional algorithm
44. [ ] when to call the planner
45. [ ] handle estimated
46. [x] move model to models
47. [x] check if measured value is in domain
48. [ ] flow graph to readme
49. [ ] maybe try to write a cdcl solver?
50. [ ] some theories? hmm...

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

| where  | what          |               docs |              tests |            benches |
| :----- | :------------ | -----------------: | -----------------: | -----------------: |
| core   | compositional | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | incremental   | :heavy_check_mark: | :heavy_check_mark: | :white_check_mark: |
| core   | items         | :heavy_check_mark: | :heavy_check_mark: | :white_check_mark: |
| core   | parameterized | :heavy_check_mark: | :heavy_check_mark: | :white_check_mark: |
| core   | predicates    | :heavy_check_mark: | :heavy_check_mark: | :white_check_mark: |
| models | blocksworld   | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| models | dummy_robot   | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | dummy         | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | node          | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | publisher     | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | receiver      | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | sender        | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | state         | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | ticker        | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | core          | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | general       | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | runner        | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| main   | main          | :white_check_mark: | :white_check_mark: | :white_check_mark: |

## thoughts
1. using a pddl instance and domain, to come up with a valid plan, negatives have to be
instantiated for the initial state. Otherwise init == goal in step 0. If a pddl model
should be run online with replanning, either generating these negatives in every step
would be necessary or, better, invariants would have to be included.
2. 