import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_CheckMethod } from '../zone-presets/zone-presets-check-method.enum';

@InputType()
export class EnumZonePresets_CheckMethodFieldUpdateOperationsInput {

    @Field(() => ZonePresets_CheckMethod, {nullable:true})
    set?: keyof typeof ZonePresets_CheckMethod;
}
