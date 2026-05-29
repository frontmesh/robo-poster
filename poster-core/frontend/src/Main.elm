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
                            Api.getPosts GotPosts

                        Accounts ->
                            Api.getAccounts GotAccounts

                        Calendar ->
                            Api.getCalendar GotCalendar

                        _ ->
                            Cmd.none
            in
            ( { model | page = page }, cmd )

        LoginEmail _ ->
            ( model, Cmd.none )

        LoginPassword _ ->
            ( model, Cmd.none )

        DoLogin ->
            ( model, Cmd.none )

        LoginResult _ ->
            ( model, Cmd.none )

        RegisterEmail _ ->
            ( model, Cmd.none )

        RegisterPassword _ ->
            ( model, Cmd.none )

        DoRegister ->
            ( model, Cmd.none )

        RegisterResult _ ->
            ( model, Cmd.none )

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
                            Api.generateContent "" prompt compose.platform GotGeneratedContent

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
                            Api.createPost "" accountId compose.content compose.scheduledAt compose.platform PostCreated

                        Nothing ->
                            Cmd.none
            in
            ( model, cmd )

        PostCreated result ->
            case result of
                Ok _ ->
                    ( { model | page = Dashboard }, Api.getPosts GotPosts )

                Err _ ->
                    ( { model | error = Just "Failed to create post" }, Cmd.none )

        PublishPost postId ->
            ( model, Api.publishPost "" postId PostPublished )

        PostPublished result ->
            case result of
                Ok _ ->
                    ( model, Api.getPosts GotPosts )

                Err _ ->
                    ( { model | error = Just "Failed to publish post" }, Cmd.none )

        ConnectAccount ->
            ( model, Api.connectAccount GotOAuthUrl )

        GotOAuthUrl result ->
            case result of
                Ok url ->
                    ( model, Http.get { url = url, expect = Http.expectWhatever (\_ -> DismissError) } )

                Err _ ->
                    ( { model | error = Just "Failed to start OAuth" }, Cmd.none )

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
        , div [ class "navbar-menu" ]
            [ navLink Dashboard "Dashboard"
            , navLink Composer "Composer"
            , navLink Calendar "Calendar"
            , navLink Accounts "Accounts"
            , navLink Analytics "Analytics"
            ]
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
            viewLogin

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


viewLogin : Html Msg
viewLogin =
    div [ class "page login-page" ]
        [ h1 [] [ text "Poster" ]
        , p [] [ text "Marketing automation for Threads & Instagram" ]
        , div [ class "login-form" ]
            [ input [ placeholder "Email", type_ "email" ] []
            , input [ placeholder "Password", type_ "password" ] []
            , button [ onClick DoLogin ] [ text "Login" ]
            , button [ onClick DoRegister ] [ text "Register" ]
            ]
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "page" ]
        [ h1 [] [ text "Dashboard" ]
        , div [ class "posts-list" ]
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
        , div [ class "post-actions" ]
            [ button [ onClick (PublishPost post.id) ] [ text "Publish" ]
            ]
        ]


viewAccounts : Model -> Html Msg
viewAccounts model =
    div [ class "page" ]
        [ h1 [] [ text "Accounts" ]
        , button [ onClick ConnectAccount ] [ text "+ Connect Instagram" ]
        , div [ class "accounts-list" ]
            (List.map viewAccountCard model.accounts)
        ]


viewAccountCard : Account -> Html Msg
viewAccountCard account =
    div [ class "account-card" ]
        [ div [ class "account-username" ] [ text account.username ]
        , div [ class "account-provider" ] [ text account.provider ]
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
            [ textarea
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
                , button [ onClick CreatePost ] [ text "Save Draft" ]
                ]
            , div [ class "ai-section" ]
                [ h3 [] [ text "AI Assistant" ]
                , input
                    [ placeholder "Describe what to write about..."
                    , onInput UpdateAiPrompt
                    ]
                    []
                , button [ onClick GenerateContent ] [ text "Generate" ]
                ]
            ]
        ]


viewCalendar : Model -> Html Msg
viewCalendar model =
    div [ class "page" ]
        [ h1 [] [ text "Content Calendar" ]
        , div [ class "calendar" ]
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
