import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { Zones_Units } from './zones-units.enum';

@InputType()
export class EnumZones_UnitsFieldUpdateOperationsInput {

    @Field(() => Zones_Units, {nullable:true})
    set?: keyof typeof Zones_Units;
}
