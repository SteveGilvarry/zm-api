import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Events_Orientation } from '../events/events-orientation.enum';
import { NestedEnumEvents_OrientationFilter } from './nested-enum-events-orientation-filter.input';

@InputType()
export class EnumEvents_OrientationFilter {

    @Field(() => Events_Orientation, {nullable:true})
    equals?: keyof typeof Events_Orientation;

    @Field(() => [Events_Orientation], {nullable:true})
    in?: Array<keyof typeof Events_Orientation>;

    @Field(() => [Events_Orientation], {nullable:true})
    notIn?: Array<keyof typeof Events_Orientation>;

    @Field(() => NestedEnumEvents_OrientationFilter, {nullable:true})
    not?: NestedEnumEvents_OrientationFilter;
}
