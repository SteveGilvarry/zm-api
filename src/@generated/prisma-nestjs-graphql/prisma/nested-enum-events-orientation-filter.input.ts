import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Orientation } from '../events/events-orientation.enum';

@InputType()
export class NestedEnumEvents_OrientationFilter {

    @Field(() => Events_Orientation, {nullable:true})
    equals?: keyof typeof Events_Orientation;

    @Field(() => [Events_Orientation], {nullable:true})
    in?: Array<keyof typeof Events_Orientation>;

    @Field(() => [Events_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Events_Orientation>;

    @Field(() => NestedEnumEvents_OrientationFilter, {nullable:true})
    not?: NestedEnumEvents_OrientationFilter;
}
