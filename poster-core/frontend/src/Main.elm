module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Types exposing (..)
import Api


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Login
      , token = Nothing
      , userId = Nothing
      , posts = []
      , accounts = []
      , calendar = []
      , compose =
            { content = ""
            , selectedAccount = Nothing
            , scheduledAt = Nothing
            , platform = "threads"
            , aiPrompt = Nothing
            , aiGenerating = False
            }
      , error = Nothing
      , loginEmail = ""
      , loginPassword = ""
      , registerEmail = ""
      , registerPassword = ""
      , loading = False
      }
    , Cmd.none
    )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Navigate page ->
            let
                cmd =
                    case page of
                        Dashboard ->
                            Api.getPosts model.token GotPosts

                        Accounts ->
                            Api.getAccounts model.token GotAccounts

                        Calendar ->
                            Api.getCalendar model.token GotCalendar

                        _ ->
                            Cmd.none
            in
            ( { model | page = page }, cmd )

        LoginEmail email ->
            ( { model | loginEmail = email }, Cmd.none )

        LoginPassword password ->
            ( { model | loginPassword = password }, Cmd.none )

        DoLogin ->
            ( { model | loading = True }
            , Api.loginRequest model.loginEmail model.loginPassword LoginResult
            )

        LoginResult result ->
            case result of
                Ok token ->
                    ( { model
                        | token = Just token
                        , page = Dashboard
                        , loading = False
                        , loginEmail = ""
                        , loginPassword = ""
                      }
                    , Api.getPosts (Just token) GotPosts
                    )

                Err _ ->
                    ( { model
                        | error = Just "Invalid email or password"
                        , loading = False
                      }
                    , Cmd.none
                    )

        RegisterEmail email ->
            ( { model | registerEmail = email }, Cmd.none )

        RegisterPassword password ->
            ( { model | registerPassword = password }, Cmd.none )

        DoRegister ->
            ( { model | loading = True }
            , Api.registerRequest model.registerEmail model.registerPassword RegisterResult
            )

        RegisterResult result ->
            case result of
                Ok token ->
                    ( { model
                        | token = Just token
                        , page = Dashboard
                        , loading = False
                        , registerEmail = ""
                        , registerPassword = ""
                      }
                    , Api.getPosts (Just token) GotPosts
                    )

                Err _ ->
                    ( { model
                        | error = Just "Registration failed. Email may already be in use."
                        , loading = False
                      }
                    , Cmd.none
                    )

        Logout ->
            ( { model
                | token = Nothing
                , userId = Nothing
                , page = Login
                , posts = []
                , accounts = []
                , calendar = []
              }
            , Cmd.none
            )

        GotPosts result ->
            case result of
                Ok posts ->
                    ( { model | posts = posts }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load posts" }, Cmd.none )

        GotAccounts result ->
            case result of
                Ok accounts ->
                    ( { model | accounts = accounts }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load accounts" }, Cmd.none )

        GotCalendar result ->
            case result of
                Ok calendar ->
                    ( { model | calendar = calendar }, Cmd.none )

                Err _ ->
                    ( { model | error = Just "Failed to load calendar" }, Cmd.none )

        UpdateComposeContent content ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | content = content } }, Cmd.none )

        UpdateComposePlatform platform ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | platform = platform } }, Cmd.none )

        UpdateComposeAccount accountId ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | selectedAccount = Just accountId } }, Cmd.none )

        UpdateAiPrompt prompt ->
            let
                compose =
                    model.compose
            in
            ( { model | compose = { compose | aiPrompt = Just prompt } }, Cmd.none )

        GenerateContent ->
            let
                compose =
                    model.compose

                cmd =
                    case compose.aiPrompt of
                        Just prompt ->
                            Api.generateContent model.token prompt compose.platform GotGeneratedContent

                        Nothing ->
                            Cmd.none
            in
            ( { model | compose = { compose | aiGenerating = True } }, cmd )

        GotGeneratedContent result ->
            let
                compose =
                    model.compose
            in
            case result of
                Ok content ->
                    ( { model | compose = { compose | content = content, aiGenerating = False } }, Cmd.none )

                Err _ ->
                    ( { model | compose = { compose | aiGenerating = False }, error = Just "Failed to generate content" }, Cmd.none )

        CreatePost ->
            let
                compose =
                    model.compose

                cmd =
                    case compose.selectedAccount of
                        Just accountId ->
                            Api.createPost model.token accountId compose.content compose.scheduledAt compose.platform PostCreated

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        PostCreated result ->
            case result of
                Ok _ ->
                    ( { model | page = Dashboard }, Api.getPosts model.token GotPosts )

                Err _ ->
                    ( { model | error = Just "Failed to create post" }, Cmd.none )

        PublishPost postId ->
            ( model, Api.publishPost model.token postId PostPublished )

        PostPublished result ->
            case result of
                Ok _ ->
                    ( model, Api.getPosts model.token GotPosts )

                Err _ ->
                    ( { model | error = Just "Failed to publish post" }, Cmd.none )

        ConnectAccount ->
            ( model, Api.connectAccount model.token GotOAuthUrl )

        GotOAuthUrl result ->
            case result of
                Ok url ->
                    ( model, Http.get { url = url, expect = Http.expectWhatever (\_ -> DismissError) } )

                Err _ ->
                    ( { model | error = Just "Failed to start OAuth" }, Cmd.none )

        DeleteAccount accountId ->
            ( model, Api.deleteAccount model.token accountId AccountDeleted )

        AccountDeleted result ->
            case result of
                Ok _ ->
                    ( model, Api.getAccounts model.token GotAccounts )

                Err _ ->
                    ( { model | error = Just "Failed to delete account" }, Cmd.none )

        DismissError ->
            ( { model | error = Nothing }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewNavbar model
        , viewError model.error
        , viewPage model
        ]


viewNavbar : Model -> Html Msg
viewNavbar model =
    nav [ class "navbar" ]
        [ div [ class "navbar-brand" ] [ text "Poster" ]
        , if model.token /= Nothing then
            div [ class "navbar-menu" ]
                [ navLink Dashboard "Dashboard"
                , navLink Composer "Composer"
                , navLink Calendar "Calendar"
                , navLink Accounts "Accounts"
                , navLink Analytics "Analytics"
                , a [ class "nav-link", onClick Logout ] [ text "Logout" ]
                ]

          else
            text ""
        ]


navLink : Page -> String -> Html Msg
navLink page label =
    a [ class "nav-link", onClick (Navigate page) ] [ text label ]


viewError : Maybe String -> Html Msg
viewError error =
    case error of
        Just msg ->
            div [ class "error-banner" ]
                [ text msg
                , button [ onClick DismissError ] [ text "x" ]
                ]

        Nothing ->
            text ""


viewPage : Model -> Html Msg
viewPage model =
    case model.page of
        Login ->
            viewLogin model

        Dashboard ->
            viewDashboard model

        Accounts ->
            viewAccounts model

        Composer ->
            viewComposer model

        Calendar ->
            viewCalendar model

        Analytics ->
            viewAnalytics

        Settings ->
            viewSettings


viewLogin : Model -> Html Msg
viewLogin model =
    div [ class "page login-page" ]
        [ h1 [] [ text "Poster" ]
        , p [ class "subtitle" ] [ text "Marketing automation for Threads & Instagram" ]
        , div [ class "login-form" ]
            [ h2 [] [ text "Login" ]
            , input
                [ placeholder "Email"
                , type_ "email"
                , value model.loginEmail
                , onInput LoginEmail
                ]
                []
            , input
                [ placeholder "Password"
                , type_ "password"
                , value model.loginPassword
                , onInput LoginPassword
                ]
                []
            , button
                [ onClick DoLogin
                , disabled model.loading
                ]
                [ text (if model.loading then "Logging in..." else "Login") ]
            , div [ class "divider" ] [ text "or" ]
            , h2 [] [ text "Register" ]
            , input
                [ placeholder "Email"
                , type_ "email"
                , value model.registerEmail
                , onInput RegisterEmail
                ]
                []
            , input
                [ placeholder "Password (min 8 characters)"
                , type_ "password"
                , value model.registerPassword
                , onInput RegisterPassword
                ]
                []
            , button
                [ onClick DoRegister
                , disabled model.loading
                ]
                [ text (if model.loading then "Registering..." else "Register") ]
            ]
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "page" ]
        [ h1 [] [ text "Dashboard" ]
        , if List.isEmpty model.posts then
            p [] [ text "No posts yet. Create one in the Composer." ]

          else
            div [ class "posts-list" ]
                (List.map viewPostCard model.posts)
        ]


viewPostCard : Post -> Html Msg
viewPostCard post =
    div [ class "post-card" ]
        [ div [ class "post-content" ] [ text post.content ]
        , div [ class "post-meta" ]
            [ span [ class "post-status" ] [ text post.status ]
            , span [ class "post-platform" ] [ text post.platform ]
            ]
        , if post.status == "draft" || post.status == "scheduled" then
            div [ class "post-actions" ]
                [ button [ onClick (PublishPost post.id) ] [ text "Publish Now" ]
                ]

          else
            text ""
        ]


viewAccounts : Model -> Html Msg
viewAccounts model =
    div [ class "page" ]
        [ h1 [] [ text "Accounts" ]
        , button [ onClick ConnectAccount, class "btn-primary" ] [ text "+ Connect Instagram / Threads" ]
        , if List.isEmpty model.accounts then
            div [ class "empty-state" ]
                [ p [] [ text "No accounts connected." ]
                , p [] [ text "Connect your Instagram Business account to start posting." ]
                ]

          else
            div [ class "accounts-list" ]
                (List.map viewAccountCard model.accounts)
        ]


viewAccountCard : Account -> Html Msg
viewAccountCard account =
    div [ class "account-card" ]
        [ div [ class "account-header" ]
            [ div [ class "account-info" ]
                [ div [ class "account-username" ] [ text account.username ]
                , div [ class "account-provider" ] [ text account.provider ]
                ]
            , button
                [ class "btn-danger"
                , onClick (DeleteAccount account.id)
                ]
                [ text "Disconnect" ]
            ]
        , div [ class "account-details" ]
            [ div [] [ text ("User ID: " ++ account.providerUserId) ]
            , case account.tokenExpiresAt of
                Just expires ->
                    div [] [ text ("Token expires: " ++ expires) ]

                Nothing ->
                    text ""
            ]
        ]


viewComposer : Model -> Html Msg
viewComposer model =
    let
        compose =
            model.compose
    in
    div [ class "page" ]
        [ h1 [] [ text "Compose Post" ]
        , div [ class "composer" ]
            [ if List.isEmpty model.accounts then
                div [ class "warning" ]
                    [ text "Connect an account first in the "
                    , a [ onClick (Navigate Accounts) ] [ text "Accounts" ]
                    , text " page."
                    ]

              else
                div [ class "form-group" ]
                    [ label [] [ text "Account" ]
                    , select [ onInput UpdateComposeAccount ]
                        (option [ value "" ] [ text "Select account..." ]
                            :: List.map
                                (\a ->
                                    option [ value a.id ]
                                        [ text (a.username ++ " (" ++ a.provider ++ ")") ]
                                )
                                model.accounts
                        )
                    ]
            , textarea
                [ placeholder "What's on your mind?"
                , value compose.content
                , onInput UpdateComposeContent
                ]
                []
            , div [ class "composer-options" ]
                [ select [ onInput UpdateComposePlatform ]
                    [ option [ value "threads" ] [ text "Threads" ]
                    , option [ value "instagram" ] [ text "Instagram" ]
                    ]
                , button [ onClick CreatePost, disabled (compose.selectedAccount == Nothing) ] [ text "Save Draft" ]
                ]
            , div [ class "ai-section" ]
                [ h3 [] [ text "AI Assistant" ]
                , input
                    [ placeholder "Describe what to write about..."
                    , onInput UpdateAiPrompt
                    ]
                    []
                , button
                    [ onClick GenerateContent
                    , disabled compose.aiGenerating
                    ]
                    [ text (if compose.aiGenerating then "Generating..." else "Generate") ]
                ]
            ]
        ]


viewCalendar : Model -> Html Msg
viewCalendar model =
    div [ class "page" ]
        [ h1 [] [ text "Content Calendar" ]
        , if List.isEmpty model.calendar then
            p [] [ text "No scheduled posts." ]

          else
            div [ class "calendar" ]
                (List.map viewCalendarDay model.calendar)
        ]


viewCalendarDay : CalendarDay -> Html Msg
viewCalendarDay day =
    div [ class "calendar-day" ]
        [ h3 [] [ text day.date ]
        , div [ class "calendar-posts" ]
            (List.map viewPostCard day.posts)
        ]


viewAnalytics : Html Msg
viewAnalytics =
    div [ class "page" ]
        [ h1 [] [ text "Analytics" ]
        , p [] [ text "Connect an account to see analytics." ]
        ]


viewSettings : Html Msg
viewSettings =
    div [ class "page" ]
        [ h1 [] [ text "Settings" ]
        , p [] [ text "Settings coming soon." ]
        ]
