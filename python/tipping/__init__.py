from ._lib_tipping import sum_as_string as _sum_as_string


__doc__ = _lib_tipping.__doc__
if hasattr(_lib_tipping, "__all__"):
    __all__ = _lib_tipping.__all__

    
def sum_as_string(a, b):
    return _sum_as_string(a, b)

sum_as_string.__doc__ = _sum_as_string.__doc__
