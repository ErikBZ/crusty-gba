print(f"o1\to2\tres\tAddV\tSubV")
for i in range(8):
    o1 = (i & 1) == 1
    o2 = (i >> 1 & 1) == 1
    res = (i >> 2 & 1) == 1
    add_v = (o1 == o2) and (o1 != res)
    sub_v = (o2 == res) and (o1 != res)
    print(f"{o1}\t{o2}\t{res}\t{add_v}\t{sub_v}")

