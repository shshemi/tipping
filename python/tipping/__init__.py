from ._lib_tipping import token_independency_clusters as _token_independency_clusters
from ._lib_tipping import TokenFilter as _TokenFilter


__doc__ = _lib_tipping.__doc__
if hasattr(_lib_tipping, "__all__"):
    __all__ = _lib_tipping.__all__

def token_independency_clusters(
    messages: [str],
    threshold: float = 0.5,
    special_whites: [str] = None,
    special_black: [str] =None,
    symbols: str = "()[]{}=,*",
    keep_alphabetic: bool= True,
    keep_numeric: bool = False,
    keep_impure: bool = False,
) -> [([str], set[int])]:
    """ Parse the input list of messages into multiple clusters according to their key tokens.

    ### Arguments:
        messages ([str]): a list of message for parsing.
        threshod (float): a floating number between `0.0` and `1.0` where token dependencies above it
        are considered as dependent. Default = `0.5`
        special_whites ([str]): a list of regexes that should never be recognized as parameters.
        Default = `None`
        special_black ([str]): a list of regexes that always shoud be recognized as parameters.
        Default = `None`
        symbols (str): a string where each character is a symbol. Default = `'()[]:,=*.'`
        keep_alphabetic (bool): a boolean indicating if alphabetic tokens should be kept for
        interdepency computations. Default = `True`
        keep_numeric (bool): a boolean indicating if numeric tokens should be kept for
        interdepency computations. Default = `False`
        keep_impure (bool): a boolean indicating if impure tokens should be kept for interdepency
        computations. Default = `False`


    ### Returns:
        [([str], set[int])]: A list of tuples where for each tuple the first element is the key
        tokens shared among all memebers of the clusters and the second element is the indices
        of the members within the message list
    """
    if special_black is None:
        special_black = []

    if special_whites is None:
        special_whites = []

    filter = _TokenFilter(keep_alphabetic, keep_numeric, keep_impure)
    return _token_independency_clusters(messages, threshold, special_whites, special_black, symbols, filter)


