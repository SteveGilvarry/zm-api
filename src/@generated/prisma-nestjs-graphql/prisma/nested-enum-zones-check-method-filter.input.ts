import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_CheckMethod } from '../zones/zones-check-method.enum';

@InputType()
export class NestedEnumZones_CheckMethodFilter {

    @Field(() => Zones_CheckMethod, {nullable:true})
    equals?: keyof typeof Zones_CheckMethod;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    in?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => [Zones_CheckMethod], {nullable:true})
    notIn?: Array<keyof typeof Zones_CheckMethod>;

    @Field(() => NestedEnumZones_CheckMethodFilter, {nullable:true})
    not?: NestedEnumZones_CheckMethodFilter;
}
