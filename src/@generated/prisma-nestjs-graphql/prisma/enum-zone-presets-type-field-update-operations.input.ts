import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { ZonePresets_Type } from './zone-presets-type.enum';

@InputType()
export class EnumZonePresets_TypeFieldUpdateOperationsInput {

    @Field(() => ZonePresets_Type, {nullable:true})
    set?: keyof typeof ZonePresets_Type;
}
