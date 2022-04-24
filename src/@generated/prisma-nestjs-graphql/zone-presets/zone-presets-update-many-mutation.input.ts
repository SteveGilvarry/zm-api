import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { StringFieldUpdateOperationsInput } from '../prisma/string-field-update-operations.input';
import { EnumZonePresets_TypeFieldUpdateOperationsInput } from '../prisma/enum-zone-presets-type-field-update-operations.input';
import { EnumZonePresets_UnitsFieldUpdateOperationsInput } from '../prisma/enum-zone-presets-units-field-update-operations.input';
import { EnumZonePresets_CheckMethodFieldUpdateOperationsInput } from '../prisma/enum-zone-presets-check-method-field-update-operations.input';
import { NullableIntFieldUpdateOperationsInput } from '../prisma/nullable-int-field-update-operations.input';
import { IntFieldUpdateOperationsInput } from '../prisma/int-field-update-operations.input';

@InputType()
export class ZonePresetsUpdateManyMutationInput {

    @Field(() => StringFieldUpdateOperationsInput, {nullable:true})
    Name?: StringFieldUpdateOperationsInput;

    @Field(() => EnumZonePresets_TypeFieldUpdateOperationsInput, {nullable:true})
    Type?: EnumZonePresets_TypeFieldUpdateOperationsInput;

    @Field(() => EnumZonePresets_UnitsFieldUpdateOperationsInput, {nullable:true})
    Units?: EnumZonePresets_UnitsFieldUpdateOperationsInput;

    @Field(() => EnumZonePresets_CheckMethodFieldUpdateOperationsInput, {nullable:true})
    CheckMethod?: EnumZonePresets_CheckMethodFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MinPixelThreshold?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MaxPixelThreshold?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MinAlarmPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MaxAlarmPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    FilterX?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    FilterY?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MinFilterPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MaxFilterPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MinBlobPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MaxBlobPixels?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MinBlobs?: NullableIntFieldUpdateOperationsInput;

    @Field(() => NullableIntFieldUpdateOperationsInput, {nullable:true})
    MaxBlobs?: NullableIntFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    OverloadFrames?: IntFieldUpdateOperationsInput;

    @Field(() => IntFieldUpdateOperationsInput, {nullable:true})
    ExtendAlarmFrames?: IntFieldUpdateOperationsInput;
}
