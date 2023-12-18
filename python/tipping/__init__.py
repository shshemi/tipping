from typing import List, Set, Tuple
from ._lib_tipping import token_independency_clusters as _token_independency_clusters
from ._lib_tipping import TokenFilter as _TokenFilter
from ._lib_tipping import Computations as _Computation


__doc__ = _lib_tipping.__doc__
if hasattr(_lib_tipping, "__all__"):
    __all__ = _lib_tipping.__all__


def token_independency_clusters(
    messages: List[str],
    threshold: float = 0.5,
    special_whites: List[str] = None,
    special_black: List[str] = None,
    symbols: str = "()[]{}=,*",
    keep_alphabetic: bool = True,
    keep_numeric: bool = False,
    keep_impure: bool = False,
    return_templates: bool = True,
    return_masks: bool = True,
) -> Tuple[List[int | None], List[str], List[Set[str]]]:
    """Parse the input list of messages into multiple clusters according to their key tokens.

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
        return_templates (bool): a boolean indicating if template computation is required. Default = `True`
        return_masks (bool): a boolean indicating if mask computation is required. Default = `True`


    ### Returns:
        Tuple[List[int | None], List[str], List[Set[str]]]: A tuple of three element where the first list
        of optional integers where for integer values are indications of cluster ids and `None` is used
        when the cluster couldn't be identified, and the second element is the corresponding parameter mask
        for each message, and the third is an array where each element is a set of template.
    """
    if special_black is None:
        special_black = []

    if special_whites is None:
        special_whites = []

    filter = _TokenFilter(keep_alphabetic, keep_numeric, keep_impure)
    computations = _Computation(return_templates, return_masks)
    return _token_independency_clusters(
        messages,
        threshold,
        special_whites,
        special_black,
        symbols,
        filter,
        computations,
    )
