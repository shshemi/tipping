import tipping


def test_tokenizer():
    msg = "Fan fan_2 speed is set to 12.3114 on machine sys.node.fan_3 on node 12"
    special_whites = [r"fan_\d+"]
    special_blacks = [r"\d+\.\d+"]
    symbols = "."
    expected = [
            "Fan",
            " ",
            "fan_2",
            " ",
            "speed",
            " ",
            "is",
            " ",
            "set",
            " ",
            "to",
            " ",
            "12.3114",
            " ",
            "on",
            " ",
            "machine",
            " ",
            "sys",
            ".",
            "node",
            ".",
            "fan_3",
            " ",
            "on",
            " ",
            "node",
            " ",
            "12", 
    ]
    tokenizer = tipping.Tokenizer(special_whites, special_blacks, symbols)
    assert tokenizer.tokenize(msg) == expected