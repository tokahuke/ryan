# Python

## Installing `ryan`

You can install Ryan directly from PyPI using the `ryan-lang` package:
```
pip install ryan-lang
```
With this command, you will get the `ryan` Python module at its most recent version installed in your environment.

## Using `ryan`

To use `ryan`, you just need to import it and use the `from_str` method:
```python
import ryan

some_values = ryan.from_str(
    """
    let x = "a value";
    { x, y: "other value" }
    """
)
```
Or you can read a JSON value _directly_ from a file using `from_path`:
```python
import ryan

some_values = ryan.from_path("some_values.ryan")
```

### Current limitations

The python library currently only exposes functions powering basic usage. This means that more advanced features, such as custom native patter matches and custom importers are not supported. However, these are more advanced features that most people will not need to use (and are not even covered in this tutorial). Most likely, the current exposed features will suffice for your use case. This limitation is intended change in a future version of this library.
