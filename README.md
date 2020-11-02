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
5.  [ ] write some macros finally
6.  [ ] no completestate just state that holds all
7.  [ ] implemment fmt
8.  [x] remove enum and bool and have only variable
9.  [ ] use 'model' and 'instance'
10. [x] rename SET to ASS
11. [ ] ok, going towards making the pddl parser :(
12. [ ] think about how to parameterize modelling like the pddl people do
13. [x] update command vars to match measured values on refresh (raar)
14. [x] raar: act, ref and act_ref have to have the same domain and same r#type
15. [ ] where is the state? who owns the state?
16. [x] proper command line arguments
17. [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
18. [ ] the "start/stop running" should actually set the reference values     ?       
19. [x] have to add delay for dummy act             
20. [ ] add boolean variables
21. [x] add handshake kind
22. [ ] the algorithms should probably be solver agnostic
23. [x] move setup to parser
24. [ ] other command paradigm or are we done?
25. [ ] testing
26. [x] structure and modules
27. [ ] generate dummy driver from the model
28. [x] documentation, probably not done
29. [ ] dummy_value should be deeply integrated
30. [ ] proper readme
31. [x] proper get planning result
32. [ ] include readmes to benches to describe added constraints
33. [x] add a timeout
34. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean w/ invariants
    2. [ ] micro_sp inc enumerated boolean explicit (neg-pddl)
    3. [ ] micro_sp inc pure boolean
    4. [ ] micro_sp inc pure enumeration
    5. [ ] micro_sp seq enumerated boolean
    6. [ ] micro_sp seq pure boolean
    7. [ ] micro_sp seq pure enumeration
    8. [ ] incplan
35. [ ] benchmarks barman
36. [ ] benchmarks childsnack
37. [ ] benchmarks gripper
38. [ ] benchmarks hiking
39. [ ] study conversion from a pddl model to a real world "runneble" model
40. [ ] enable other solvers beside z3
41. [ ] make a more general parser for boilerplate domain ? maybe not
42. [ ] clean the warnings
43. [ ] improve runner and don't replan after every state change
44. [ ] gui needed asap
45. [ ] show current plan in gui
46. [x] look into parsing pddls: conclusion: won't work
47. [ ] fix command lifetime
48. [x] match published data to sink when fresh timeouts
49. [ ] compositional algorithm
50. [ ] when to call the planner
51. [ ] handle estimated
52. [x] move model to models
53. [x] check if measured value is in domain
54. [ ] flow graph to readme
55. [ ] maybe try to write a cdcl solver?
56. [ ] some theories? hmm...

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
    
### docs tests macros status

| where  | what          |               docs |              tests |             macros |              mdocs |             mtests |            benches |
| :----- | :------------ | -----------------: | -----------------: | -----------------: | -----------------: | -----------------: | -----------------: |
| core   | compositional | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | incremental   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | items         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | parameterized | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| core   | predicates    | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| models | blocksworld   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| models | dummy_robot   | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | dummy         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | node          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | publisher     | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | receiver      | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | sender        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | state         | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| runner | ticker        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | core          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | general       | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| utils  | runner        | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |
| main   | main          | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

## thoughts
1. using a pddl instance and domain, to come up with a valid plan, negatives have to be
instantiated for the initial state. Otherwise init == goal in step 0. If a pddl model
should be run online with replanning, either generating these negatives in every step
would be necessary or, better, invariants would have to be included.
2. 