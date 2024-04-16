mod confirm;
mod layout;
mod signin;
mod signup;

pub(crate) use {
    confirm::ConfirmationPage, layout::AuthenticatedLayout, signin::SignInPage, signup::SignUpPage,
};
