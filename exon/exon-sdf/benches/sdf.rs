// Copyright 2024 WHERE TRUE Technologies.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use exon_sdf::parse_to_record;

fn bench_parse_to_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("sdf_parse");

    let content = "
SciTegic02060916132D

50 60  0  0  0  0            999 V2000
    -5.2740    4.8598    0.0000 O   0  0
    -5.4300    3.6700    0.0000 C   0  0
    -6.8500    3.0700    0.0000 C   0  0
    -8.0800    4.0100    0.0000 C   0  0
    -9.5100    3.4300    0.0000 C   0  0
    -9.6700    1.7700    0.0000 C   0  0
    -8.3900    0.8900    0.0000 C   0  0
    -6.9800    1.4800    0.0000 C   0  0
    -5.7800    0.6200    0.0000 C   0  0
    -5.9171   -0.5721    0.0000 O   0  0
    -4.3800    1.2000    0.0000 C   0  0
    -4.2200    2.7300    0.0000 C   0  0
    -2.8300    3.3500    0.0000 C   0  0
    -1.5900    2.4100    0.0000 C   0  0
    -1.7800    0.8500    0.0000 C   0  0
    -3.1400    0.2700    0.0000 C   0  0
    -2.9600   -1.2600    0.0000 N   0  0
    -1.5000   -1.5300    0.0000 C   0  0
    -0.7400   -2.8600    0.0000 C   0  0
    -1.4800   -4.1900    0.0000 C   0  0
    -2.6797   -4.2171    0.0000 O   0  0
    -0.6900   -5.4700    0.0000 C   0  0
    -1.4800   -6.8500    0.0000 C   0  0
    -0.7400   -8.1800    0.0000 C   0  0
    0.7900   -8.1800    0.0000 C   0  0
    1.5300   -6.8500    0.0000 C   0  0
    0.7900   -5.4700    0.0000 C   0  0
    1.5300   -4.1500    0.0000 C   0  0
    2.7300   -4.1519    0.0000 O   0  0
    0.7900   -2.8200    0.0000 C   0  0
    1.5300   -1.4800    0.0000 C   0  0
    3.0000   -1.1900    0.0000 N   0  0
    3.1500    0.3200    0.0000 C   0  0
    4.3800    1.2600    0.0000 C   0  0
    5.7900    0.6800    0.0000 C   0  0
    5.9393   -0.5107    0.0000 O   0  0
    7.0400    1.6100    0.0000 C   0  0
    8.4400    1.0700    0.0000 C   0  0
    9.6800    2.0400    0.0000 C   0  0
    9.4000    3.5400    0.0000 C   0  0
    8.0000    4.0900    0.0000 C   0  0
    6.7800    3.1400    0.0000 C   0  0
    5.3800    3.7200    0.0000 C   0  0
    5.2120    4.9082    0.0000 O   0  0
    4.1900    2.7700    0.0000 C   0  0
    2.7300    3.3900    0.0000 C   0  0
    1.5100    2.4400    0.0000 C   0  0
    1.7700    0.8500    0.0000 C   0  0
    0.7400   -0.2700    0.0000 C   0  0
    -0.7700   -0.2700    0.0000 C   0  0
    1  2  2  0
    2  3  1  0
    3  4  2  0
    4  5  1  0
    5  6  2  0
    6  7  1  0
    7  8  2  0
    8  3  1  0
    8  9  1  0
    9 10  2  0
    9 11  1  0
11 12  2  0
12  2  1  0
12 13  1  0
13 14  2  0
14 15  1  0
15 16  2  0
16 11  1  0
16 17  1  0
17 18  1  0
18 19  2  0
19 20  1  0
20 21  2  0
20 22  1  0
22 23  2  0
23 24  1  0
24 25  2  0
25 26  1  0
26 27  2  0
27 22  1  0
27 28  1  0
28 29  2  0
28 30  1  0
30 19  1  0
30 31  2  0
31 32  1  0
32 33  1  0
33 34  2  0
34 35  1  0
35 36  2  0
35 37  1  0
37 38  2  0
38 39  1  0
39 40  2  0
40 41  1  0
41 42  2  0
42 37  1  0
42 43  1  0
43 44  2  0
43 45  1  0
45 34  1  0
45 46  2  0
46 47  1  0
47 48  2  0
48 33  1  0
48 49  1  0
49 31  1  0
49 50  2  0
50 15  1  0
50 18  1  0
M  END
> <canonical_smiles>
O=C1c2ccccc2C(=O)c3c1ccc4c3[nH]c5c6C(=O)c7ccccc7C(=O)c6c8[nH]c9c%10C(=O)c%11ccccc%11C(=O)c%10ccc9c8c45

> <CAS_NO>
2475-33-4

> <Source>
VITIC

> <Activity>
0

> <WDI_Name>
.

> <REFERENCE>
JUDSON, PN, COOKE, PA, DOERRER, NG, GREENE, N, HANZLIK, RP, HARDY, C, HARTMANN, A, HINCHLIFFE, D, HOLDER, J, MUELLER, L, STEGER-HARTMANN, T, ROTHFUSS, A, SMITH, M, THOMAS, K, VESSEY, JD AND ZEIGER E.
TOWARDS THE CREATION OF AN INTERNATIONAL TOXICOLOGY INFORMATION CENTRE. TOXICOLOGY 213(1-2):117-28, 2005

> <MC_Example>
0

> <MC_Pred>
0

> <DEREK_Example>
0

> <DEREK_Pred>
1

> <Molecular_Weight>
646.60212

> <Set>
CV3

$$$$
";

    group.bench_with_input(
        BenchmarkId::new("sdf_parse", "content"),
        &content,
        |b, content| {
            b.iter(|| {
                parse_to_record(content).expect("Failed to parse record");
            });
        },
    );

    group.finish();
}

criterion_group!(benches, bench_parse_to_record);
criterion_main!(benches);
