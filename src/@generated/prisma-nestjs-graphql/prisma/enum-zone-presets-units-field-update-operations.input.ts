import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Units } from './zone-presets-units.enum';

@InputType()
export class EnumZonePresets_UnitsFieldUpdateOperationsInput {

    @Field(() => ZonePresets_Units, {nullable:true})
    set?: keyof typeof ZonePresets_Units;
}
