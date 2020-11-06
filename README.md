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
2.  [ ] move params into paramproblem
3.  [ ] figure out why do tests run twice?
4.  [x] don't complicate, have parameterized
5.  [x] layered (comp(param(inc)))
6.  [x] publish complete state
7.  [ ] write some macros finally
8.  [x] no completestate just state that holds all
9.  [ ] implemment fmt
10. [x] remove enum and bool and have only variable
11. [ ] use 'model' and 'instance'
12. [x] rename SET to ASS
13. [ ] ok, going towards making the pddl parser :(
14. [ ] think about how to parameterize modelling like the pddl people do
15. [x] update command vars to match measured values on refresh (raar)
16. [x] raar: act, ref and act_ref have to have the same domain and same r#type
17. [ ] where is the state? who owns the state?
18. [x] proper command line arguments
19. [ ] need new "start/stop running" and "set goal" topics to communicate with the runner
20. [ ] the "start/stop running" should actually set the reference values     ?       
21. [x] have to add delay for dummy act             
22. [ ] add boolean variables
23. [x] add handshake kind
24. [ ] the algorithms should probably be solver agnostic
25. [x] move setup to parser
26. [ ] other command paradigm or are we done?
27. [ ] testing
28. [x] structure and modules
29. [ ] generate dummy driver from the model
30. [x] documentation, probably not done
31. [ ] dummy_value should be deeply integrated
32. [ ] proper readme
33. [x] proper get planning result
34. [ ] include readmes to benches to describe added constraints
35. [x] add a timeout
36. [ ] benchmarks blocksworld:
    1. [x] micro_sp inc enumerated boolean w/ invariants
    2. [ ] micro_sp inc enumerated boolean explicit (neg-pddl)
    3. [ ] micro_sp inc pure boolean
    4. [ ] micro_sp inc pure enumeration
    5. [ ] micro_sp seq enumerated boolean
    6. [ ] micro_sp seq pure boolean
    7. [ ] micro_sp seq pure enumeration
    8. [ ] incplan
37. [ ] benchmarks barman
38. [ ] benchmarks childsnack
39. [ ] benchmarks gripper
    1. [ ] micro_sp inc enumerated boolean w/ invariants
    2. [ ] micro_sp inc enumerated boolean explicit (neg-pddl)
40. [ ] benchmarks hiking
41. [ ] study conversion from a pddl model to a real world "runneble" model
42. [ ] enable other solvers beside z3
43. [ ] make a more general parser for boilerplate domain ? maybe not
44. [ ] clean the warnings
45. [ ] improve runner and don't replan after every state change
46. [ ] gui needed asap
47. [ ] show current plan in gui
48. [x] look into parsing pddls: conclusion: won't work
49. [ ] fix command lifetime
50. [x] match published data to sink when fresh timeouts
51. [ ] compositional algorithm
52. [ ] when to call the planner
53. [ ] handle estimated
54. [x] move model to models
55. [x] check if measured value is in domain
56. [ ] flow graph to readme
57. [ ] maybe try to write a cdcl solver?
58. [ ] some theories? hmm...

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
2. problem = model(trans and invars) + instance(init + goal)