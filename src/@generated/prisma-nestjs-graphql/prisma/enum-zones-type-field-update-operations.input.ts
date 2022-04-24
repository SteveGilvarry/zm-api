import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Type } from './zones-type.enum';

@InputType()
export class EnumZones_TypeFieldUpdateOperationsInput {

    @Field(() => Zones_Type, {nullable:true})
    set?: keyof typeof Zones_Type;
}
