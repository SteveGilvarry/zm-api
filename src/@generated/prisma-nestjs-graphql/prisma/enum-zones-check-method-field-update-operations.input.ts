import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_CheckMethod } from '../zones/zones-check-method.enum';

@InputType()
export class EnumZones_CheckMethodFieldUpdateOperationsInput {

    @Field(() => Zones_CheckMethod, {nullable:true})
    set?: keyof typeof Zones_CheckMethod;
}
